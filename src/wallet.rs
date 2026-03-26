use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use bdk_esplora::EsploraAsyncExt;
#[allow(deprecated)]
use bdk_wallet::SignOptions;
use bdk_wallet::Wallet;
use bdk_wallet::bitcoin::psbt::Psbt;
use bdk_wallet::bitcoin::{Address, FeeRate, Network, Txid};
use esplora_client::{Builder, Sleeper};
use gloo_timers::future::TimeoutFuture;
use send_wrapper::SendWrapper;

/// WASM-compatible sleeper using gloo-timers wrapped for Send
#[derive(Clone, Copy)]
pub struct WasmSleeper;

/// Wrapper to make TimeoutFuture implement Send (safe in single-threaded WASM)
pub struct WasmSleep(SendWrapper<TimeoutFuture>);

impl Future for WasmSleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll(cx)
    }
}

// SAFETY: WASM is single-threaded, so Send is safe
unsafe impl Send for WasmSleep {}

impl Sleeper for WasmSleeper {
    type Sleep = WasmSleep;

    fn sleep(dur: Duration) -> Self::Sleep {
        WasmSleep(SendWrapper::new(TimeoutFuture::new(dur.as_millis() as u32)))
    }
}

/// Compute the BFT threshold for n guardians
/// This matches fedimint's threshold calculation: n - (n-1)/3
pub fn bft_threshold(n: usize) -> usize {
    n - (n - 1) / 3
}

/// Build a signing descriptor from private keys (WIF) and public keys (hex).
/// Filters out empty keys; sortedmulti will sort them, so order doesn't matter.
pub fn build_descriptor(
    threshold: usize,
    private_keys: &[String],
    public_keys: &[String],
) -> String {
    let all_keys: Vec<&str> = private_keys
        .iter()
        .chain(public_keys.iter())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    format!("wsh(sortedmulti({},{}))", threshold, all_keys.join(","))
}

pub fn create_wallet(descriptor: &str, network: Network) -> Result<Wallet> {
    Wallet::create_single(descriptor.to_string())
        .network(network)
        .create_wallet_no_persist()
        .context("Failed to create wallet from descriptor")
}

pub async fn sync_wallet(wallet: &mut Wallet, esplora_url: &str) -> Result<u64> {
    let client = Builder::new(esplora_url)
        .build_async_with_sleeper::<WasmSleeper>()
        .context("Failed to build esplora client")?;

    let start_time = (js_sys::Date::now() / 1000.0) as u64;

    let request = wallet.start_full_scan_at(start_time).build();

    let update = client
        .full_scan(request, 3, 3)
        .await
        .context("Failed to scan blockchain")?;

    wallet
        .apply_update(update)
        .context("Failed to apply wallet update")?;

    Ok(wallet.balance().total().to_sat())
}

pub fn build_sweep_tx(
    wallet: &mut Wallet,
    destination: Address,
    fee_rate_sat_vb: u64,
) -> Result<Psbt> {
    let fee_rate = FeeRate::from_sat_per_vb(fee_rate_sat_vb).context("Invalid fee rate")?;

    let mut builder = wallet.build_tx();

    builder
        .drain_wallet()
        .drain_to(destination.script_pubkey())
        .fee_rate(fee_rate);

    let mut psbt = builder.finish().context("Failed to build transaction")?;

    #[allow(deprecated)]
    let finalized = wallet
        .sign(&mut psbt, SignOptions::default())
        .context("Failed to sign transaction")?;

    if !finalized {
        bail!("Transaction signing incomplete - need more signatures");
    }

    Ok(psbt)
}

pub async fn broadcast_tx(esplora_url: &str, psbt: &Psbt) -> Result<Txid> {
    let client = Builder::new(esplora_url)
        .build_async_with_sleeper::<WasmSleeper>()
        .context("Failed to build esplora client")?;

    let tx = psbt
        .clone()
        .extract_tx()
        .context("Failed to extract transaction from PSBT")?;

    let txid = tx.compute_txid();

    client
        .broadcast(&tx)
        .await
        .context("Failed to broadcast transaction")?;

    Ok(txid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bft_threshold() {
        assert_eq!(bft_threshold(4), 3); // 4 - (4-1)/3 = 4 - 1 = 3
        assert_eq!(bft_threshold(5), 4); // 5 - (5-1)/3 = 5 - 1 = 4
        assert_eq!(bft_threshold(6), 5); // 6 - (6-1)/3 = 6 - 1 = 5
        assert_eq!(bft_threshold(7), 5); // 7 - (7-1)/3 = 7 - 2 = 5
    }
}
