# Bug Report: opencode-usage-companion

**Date:** 2026-02-01
**Status:** Open
**Severity:** Critical to Low

This document outlines bugs and improvement areas identified during a code review of the `opencode-usage-companion` repository.

## 1. Critical Logic Bug: Gemini Auth Inconsistency

*   **Location:** `src/auth.rs` (lines 144-150) vs `src/providers/gemini.rs` (lines 330-334)
*   **Description:**
    The `AuthManager::is_provider_configured` method in `auth.rs` considers Gemini "configured" if *either* `antigravity-accounts.json` exists OR `auth.json` contains a Google token (`has_google_oauth`).
    However, the `GeminiProvider::fetch` implementation in `gemini.rs` **only** supports reading from `antigravity-accounts.json`. It does not verify or use the token from `auth.json`.
*   **Symptom:**
    Users who are authenticated with OpenCode (via `auth.json`) but do not have the Antigravity plugin configured (`antigravity-accounts.json`) will see the tool attempt to query Gemini and fail with the error: `Warning: gemini failed: gemini (no antigravity accounts found)`. The tool incorrectly reports Gemini as configured.
*   **Proposed Fix:**
    Modify `src/auth.rs` to remove the `has_google_oauth` check for Gemini. `is_configured` should return true *only* if `antigravity-accounts.json` exists, aligning with the actual capabilities of the `GeminiProvider`.

## 2. Critical Stability: Fragile Auth File Parsing

*   **Location:** `src/auth.rs` (lines 20-28, `OAuthToken` struct)
*   **Description:**
    The `OAuthToken` struct makes the `refresh` and `expires` fields mandatory (non-Option types).
    ```rust
    pub struct OAuthToken {
        pub refresh: String, // Mandatory
        pub expires: i64,    // Mandatory
        // ...
    }
    ```
    However, in `auth.json`, these fields may not always be present (e.g., for Personal Access Tokens, simple OAuth flows, or if a provider implementation changes).
*   **Symptom:**
    If `auth.json` contains *any* provider entry that lacks a `refresh` or `expires` field (even for a provider `ocu` isn't querying), `read_opencode_auth` will fail to parse the entire file. This causes the tool to crash or report "No AI providers configured" for *all* providers.
*   **Proposed Fix:**
    Update `src/auth.rs` to make `refresh` and `expires` optional:
    ```rust
    pub refresh: Option<String>,
    pub expires: Option<i64>,
    ```

## 3. Potential Crash: Unsafe Status Calculation

*   **Location:** `src/providers/mod.rs` (line 122, inside `ProviderData::status`)
*   **Description:**
    The code uses `unwrap()` on the result of `partial_cmp` for floating-point numbers:
    ```rust
    .min_by(|a, b| a.partial_cmp(b).unwrap())
    ```
    Floating-point comparisons return `None` if values are `NaN`. While unlikely given the upstream parsing logic (which usually rejects `NaN` in JSON), relying on `unwrap()` is unsafe practice in Rust.
*   **Symptom:**
    If the internal quota calculation ever produces `NaN` (e.g., via arithmetic operations on API data), the CLI tool will panic and crash.
*   **Proposed Fix:**
    Use safe comparison logic that handles `None` (NaN), such as treating `NaN` as equal or filtering values first:
    ```rust
    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    ```

## 4. UX Issue: Gemini Default Quota Logic

*   **Location:** `src/providers/gemini.rs` (line 230)
*   **Description:**
    When parsing the API response, if `remainingFraction` is missing, the code defaults to `0.0`:
    ```rust
    let remaining_fraction = quota_info.remaining_fraction.unwrap_or(0.0)
    ```
*   **Symptom:**
    If the Google API response format changes or momentarily omits this field, the tool will report **0% remaining** (Quota Exhausted) to the user. This is a "fail-closed" behavior that causes false alarms.
*   **Proposed Fix:**
    Change the default to `1.0` (100% remaining) or introduce an `Option` to display "Unknown" instead of "0%". Defaulting to 100% is generally less disruptive for a monitoring tool unless confirmed otherwise.

## 5. UX Issue: Copilot Silent Failures

*   **Location:** `src/providers/copilot.rs` (lines 23-93, `fetch_billing_data`)
*   **Description:**
    The `fetch_billing_data` method swallows *all* errors (network issues, 403 Forbidden, API changes) and returns `None`:
    ```rust
    .send().await.ok()?; // Returns None on error
    ```
*   **Symptom:**
    If the user's token lacks the specific scope required for billing data, or if the API fails, the tool simply shows no overage information. The user has no way of knowing if they have 0 overages or if the check failed.
*   **Proposed Fix:**
    Update `fetch_billing_data` to return a `Result` or log a warning to stderr (`eprintln!`) when an API call fails with a specific error code (like 403), so the user knows *why* data is missing.
