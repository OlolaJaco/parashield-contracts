# Product Category Registry Specification

## Overview
Parashield utilizes category-level risk pools to manage risk exposure across different product domains (e.g. crop insurance, flight delays, disaster coverage).
Previously, product creation did not enforce that product categories were registered in a protocol registry, risking bypass of category-level risk pool limits and parameters (#682).

## Category Registry Architecture

### 1. Storage Layout
- `StorageKey::RegisteredCategory(Symbol)`: Boolean flag mapping category name to registration status.
- `StorageKey::CategoryRegistry`: Vector of symbols (`Vec<Symbol>`) listing all active registered categories.

### 2. Initialization and Default Categories
When `PolicyEngine::initialize` is called, the default protocol categories are pre-registered:
- `crop`: Agricultural parametric insurance (e.g. rainfall, drought).
- `flight`: Flight delay and cancellation insurance.
- `disaster`: Catastrophic natural disasters (e.g. earthquake, hurricane).
- `health`: Parametric epidemic/health coverage.
- `defi`: DeFi protocol exploit/depeg coverage.

### 3. Administrative Interface
- `register_category(admin: Address, category: Symbol)`: Adds a new category to the registry, enabling products to be created under this category. Emits `CategoryRegistered`.
- `unregister_category(admin: Address, category: Symbol)`: Deactivates an existing category. Future product creations for this category will be rejected with `Error::InvalidCategory`. Emits `CategoryUnregistered`.
- `is_category_registered(category: Symbol) -> bool`: Public view function returning registration status.
- `get_registered_categories() -> Vec<Symbol>`: Returns all active registered categories.

### 4. Enforcement in Product Creation
During `create_product`:
```rust
if !Self::is_category_registered(env.clone(), params.category.clone()) {
    panic_with_error!(&env, Error::InvalidCategory);
}
```
Any product submitted with an unregistered category is strictly rejected with `Error::InvalidCategory` (code 36).
