# Escrow Contract (Ink!) Documentation

This document describes the **Escrow smart contract** implemented in Ink! for Substrate-based blockchains.  
It explains the available methods, events, and usage workflows.

---

## Features

- Setup and configure an escrow service
- Add escrow accounts (manager only)
- Release funds (by account owner or forced by manager)
- Open and close the escrow service
- Emits detailed events for all operations
- Checks for duplicates, maximum accounts, and open/close status

---

## Error Messages

| Error Variant | Meaning |
|---------------|---------|
| `BadOrigin` | Caller is not authorized |
| `EscrowIsClose` | Escrow service is closed |
| `EscrowAccountNotFound` | The escrow account does not exist |
| `EscrowAccountDuplicate` | Account already exists in escrow |
| `EscrowAccountMax` | Maximum number of escrow accounts reached |
| `TransferFailed` | Transfer of funds failed |

---

## Success Messages

| Success Variant | Meaning |
|-----------------|---------|
| `EscrowSetupSuccess` | Escrow setup completed |
| `EscrowCloseSuccess` | Escrow closed successfully |
| `EscrowOpenSuccess` | Escrow opened successfully |
| `EscrowAccountAdded` | Escrow account added |
| `EscrowAccountReleased` | Escrow account released |

---

## Storage Structure

- `Account`
  - `reference: u16` – unique reference for the account
  - `account: AccountId` – the user’s account address
  - `balance: u128` – escrowed balance
  - `recipient: AccountId` – destination for release
  - `status: u8` – 0 = frozen, 1 = liquid

- `Escrow`
  - `asset_id: u128` – identifier of the escrowed asset
  - `owner: AccountId` – owner of the escrow contract
  - `manager: AccountId` – manager who can add/release accounts
  - `maximum_accounts: u16` – max number of escrow accounts
  - `accounts: Vec<Account>` – list of escrow accounts
  - `status: u8` – 0 = open, 1 = closed

---

## Methods / Messages

### `new(asset_id: u128, maximum_accounts: u16)`
Creates a new escrow service. The caller becomes **owner** and **manager**.

### `default()`
Creates a default escrow service with `asset_id = 0` and `maximum_accounts = 0`.

### `setup(asset_id, manager, maximum_accounts)`
- Sets or resets the escrow configuration.
- Only the **owner** can call.
- Resets all existing accounts.
- Emits `EscrowSetupSuccess`.

### `get() -> (asset_id, owner, manager, maximum_accounts, status)`
- Returns the current configuration and status of the escrow.

### `open()`
- Opens the escrow service.
- Only **manager** can call.
- Emits `EscrowOpenSuccess`.

### `close()`
- Closes the escrow service.
- Only **manager** can call.
- Emits `EscrowCloseSuccess`.

### `add(reference, account, amount, recipient)`
- Adds a new escrow account.
- Only **manager** can call.
- Checks:
  - Escrow is open
  - Account is not a duplicate
  - Maximum accounts limit not exceeded
- Emits `EscrowAccountAdded` if successful
- Emits `EscrowAccountDuplicate` or `EscrowAccountMax` on error

### `release()`
- Called by the **account owner** to release their escrowed funds.
- Transfers funds to the account’s recipient.
- Removes the account from escrow.
- Emits `EscrowAccountReleased` on success.
- Emits `EscrowAccountNotFound` if account does not exist.
- Cannot be called if escrow is closed (`EscrowIsClose`).

### `force_release(account, amount, recipient)`
- Called by **manager** to release any escrow account.
- Transfers funds to the specified recipient.
- Removes the account from escrow.
- Emits `EscrowAccountReleased` on success.
- Emits `EscrowAccountNotFound` if account does not exist.
- Cannot be called if escrow is closed (`EscrowIsClose`).

---

## Events

All operations emit the following event:

- `EscrowEvent`
  - `operator: AccountId` – the caller of the method
  - `status: EscrowStatus`
    - `EmitSuccess(Success)` – indicates successful operation
    - `EmitError(Error)` – indicates failure

Events are emitted for transparency and audit purposes.

---

## Example Workflow

1. **Deploy Escrow Contract**
   - Create new escrow with `new(asset_id, maximum_accounts)`.

2. **Setup Escrow (owner only)**
   - Call `setup(asset_id, manager, maximum_accounts)`.

3. **Add Accounts (manager only)**
   - Call `add(reference, account_id, amount, recipient)`.

4. **Release Funds (account owner)**
   - Call `release()`.

5. **Force Release (manager only)**
   - Call `force_release(account_id, amount, recipient)`.

6. **Open / Close Escrow (manager only)**
   - Call `open()` or `close()`.

---

## Notes

- Accounts are removed using a gas-efficient method (`swap_remove`).
- Only the **owner** or **manager** can perform sensitive actions.
- Transfers may fail if funds are insufficient.
- Events provide a complete audit trail.
- Escrow status (`open`/`closed`) must be checked before performing actions.
