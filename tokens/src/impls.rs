use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::{
	fungible, fungibles,
	tokens::{Balance as BalanceT, DepositConsequence, WithdrawConsequence},
	Contains, Get,
};
use sp_arithmetic::{traits::Bounded, ArithmeticError};

pub struct Combiner<AccountId, TestKey, A, B>(sp_std::marker::PhantomData<(AccountId, TestKey, A, B)>);

impl<AccountId, TestKey, A, B> fungibles::Inspect<AccountId> for Combiner<AccountId, TestKey, A, B>
where
	TestKey: Contains<<B as fungibles::Inspect<AccountId>>::AssetId>,
	A: fungible::Inspect<AccountId, Balance = <B as fungibles::Inspect<AccountId>>::Balance>,
	B: fungibles::Inspect<AccountId>,
{
	type AssetId = <B as fungibles::Inspect<AccountId>>::AssetId;
	type Balance = <B as fungibles::Inspect<AccountId>>::Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		if TestKey::contains(&asset) {
			A::total_issuance()
		} else {
			B::total_issuance(asset)
		}
	}

	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		if TestKey::contains(&asset) {
			A::minimum_balance()
		} else {
			B::minimum_balance(asset)
		}
	}

	fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		if TestKey::contains(&asset) {
			A::balance(who)
		} else {
			B::balance(asset, who)
		}
	}

	fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
		if TestKey::contains(&asset) {
			A::reducible_balance(who, keep_alive)
		} else {
			B::reducible_balance(asset, who, keep_alive)
		}
	}

	fn can_deposit(asset: Self::AssetId, who: &AccountId, amount: Self::Balance, mint: bool) -> DepositConsequence {
		if TestKey::contains(&asset) {
			A::can_deposit(who, amount, mint)
		} else {
			B::can_deposit(asset, who, amount, mint)
		}
	}

	fn can_withdraw(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		if TestKey::contains(&asset) {
			A::can_withdraw(who, amount)
		} else {
			B::can_withdraw(asset, who, amount)
		}
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		if TestKey::contains(&asset) {
			true
		} else {
			B::asset_exists(asset)
		}
	}
}

impl<AccountId, TestKey, A, B> fungibles::Transfer<AccountId> for Combiner<AccountId, TestKey, A, B>
where
	TestKey: Contains<<B as fungibles::Inspect<AccountId>>::AssetId>,
	A: fungible::Transfer<AccountId, Balance = <B as fungibles::Inspect<AccountId>>::Balance>,
	B: fungibles::Transfer<AccountId>,
{
	fn transfer(
		asset: Self::AssetId,
		source: &AccountId,
		dest: &AccountId,
		amount: Self::Balance,
		keep_alive: bool,
	) -> Result<Self::Balance, DispatchError> {
		if TestKey::contains(&asset) {
			A::transfer(source, dest, amount, keep_alive)
		} else {
			B::transfer(asset, source, dest, amount, keep_alive)
		}
	}
}

impl<AccountId, TestKey, A, B> fungibles::Mutate<AccountId> for Combiner<AccountId, TestKey, A, B>
where
	TestKey: Contains<<B as fungibles::Inspect<AccountId>>::AssetId>,
	A: fungible::Mutate<AccountId, Balance = <B as fungibles::Inspect<AccountId>>::Balance>,
	B: fungibles::Mutate<AccountId>,
{
	fn mint_into(asset: Self::AssetId, dest: &AccountId, amount: Self::Balance) -> DispatchResult {
		if TestKey::contains(&asset) {
			A::mint_into(dest, amount)
		} else {
			B::mint_into(asset, dest, amount)
		}
	}

	fn burn_from(
		asset: Self::AssetId,
		dest: &AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		if TestKey::contains(&asset) {
			A::burn_from(dest, amount)
		} else {
			B::burn_from(asset, dest, amount)
		}
	}
}

pub trait ConvertBalance<A: Bounded, B: Bounded> {
	type AssetId;
	fn convert_balance(amount: A, asset_id: Self::AssetId) -> Result<B, ArithmeticError>;
	fn convert_balance_back(amount: B, asset_id: Self::AssetId) -> Result<A, ArithmeticError>;

	fn convert_balance_saturated(amount: A, asset_id: Self::AssetId) -> B {
		Self::convert_balance(amount, asset_id).unwrap_or_else(|e| match e {
			ArithmeticError::Overflow => B::max_value(),
			ArithmeticError::Underflow => B::min_value(),
			ArithmeticError::DivisionByZero => B::max_value(),
		})
	}
	fn convert_balance_back_saturated(amount: B, asset_id: Self::AssetId) -> A {
		Self::convert_balance_back(amount, asset_id).unwrap_or_else(|e| match e {
			ArithmeticError::Overflow => A::max_value(),
			ArithmeticError::Underflow => A::min_value(),
			ArithmeticError::DivisionByZero => A::max_value(),
		})
	}
}

pub struct Mapper<AccountId, T, C, B, GetCurrencyId>(sp_std::marker::PhantomData<(AccountId, T, C, B, GetCurrencyId)>);
impl<AccountId, T, C, B, GetCurrencyId> fungible::Inspect<AccountId> for Mapper<AccountId, T, C, B, GetCurrencyId>
where
	T: fungibles::Inspect<AccountId>,
	C: ConvertBalance<
		<T as fungibles::Inspect<AccountId>>::Balance,
		B,
		AssetId = <T as fungibles::Inspect<AccountId>>::AssetId,
	>,
	B: BalanceT,
	GetCurrencyId: Get<<T as fungibles::Inspect<AccountId>>::AssetId>,
{
	type Balance = B;

	fn total_issuance() -> Self::Balance {
		C::convert_balance_saturated(T::total_issuance(GetCurrencyId::get()), GetCurrencyId::get())
	}

	fn minimum_balance() -> Self::Balance {
		C::convert_balance_saturated(T::minimum_balance(GetCurrencyId::get()), GetCurrencyId::get())
	}

	fn balance(who: &AccountId) -> Self::Balance {
		C::convert_balance_saturated(T::balance(GetCurrencyId::get(), who), GetCurrencyId::get())
	}

	fn reducible_balance(who: &AccountId, keep_alive: bool) -> Self::Balance {
		C::convert_balance_saturated(
			T::reducible_balance(GetCurrencyId::get(), who, keep_alive),
			GetCurrencyId::get(),
		)
	}

	fn can_deposit(who: &AccountId, amount: Self::Balance, mint: bool) -> DepositConsequence {
		let amount = C::convert_balance_back(amount, GetCurrencyId::get());
		let amount = match amount {
			Ok(amount) => amount,
			Err(_) => return DepositConsequence::Overflow,
		};
		T::can_deposit(GetCurrencyId::get(), who, amount, mint)
	}

	fn can_withdraw(who: &AccountId, amount: Self::Balance) -> WithdrawConsequence<Self::Balance> {
		use WithdrawConsequence::*;

		let amount = C::convert_balance_back(amount, GetCurrencyId::get());
		let amount = match amount {
			Ok(amount) => amount,
			Err(ArithmeticError::Overflow) => return Overflow,
			Err(ArithmeticError::Underflow) => return Underflow,
			Err(ArithmeticError::DivisionByZero) => return Overflow,
		};

		let res = T::can_withdraw(GetCurrencyId::get(), who, amount);
		match res {
			WithdrawConsequence::ReducedToZero(b) => {
				WithdrawConsequence::ReducedToZero(C::convert_balance_saturated(b, GetCurrencyId::get()))
			}
			NoFunds => NoFunds,
			WouldDie => WouldDie,
			UnknownAsset => UnknownAsset,
			Underflow => Underflow,
			Overflow => Overflow,
			Frozen => Frozen,
			Success => Success,
		}
	}
}

impl<AccountId, T, C, B, GetCurrencyId> fungible::Transfer<AccountId> for Mapper<AccountId, T, C, B, GetCurrencyId>
where
	T: fungibles::Transfer<AccountId, Balance = B>,
	C: ConvertBalance<
		<T as fungibles::Inspect<AccountId>>::Balance,
		B,
		AssetId = <T as fungibles::Inspect<AccountId>>::AssetId,
	>,
	B: BalanceT,
	GetCurrencyId: Get<<T as fungibles::Inspect<AccountId>>::AssetId>,
{
	fn transfer(source: &AccountId, dest: &AccountId, amount: B, keep_alive: bool) -> Result<B, DispatchError> {
		T::transfer(
			GetCurrencyId::get(),
			source,
			dest,
			C::convert_balance_back(amount, GetCurrencyId::get())?,
			keep_alive,
		)
	}
}

impl<AccountId, T, C, B, GetCurrencyId> fungible::Mutate<AccountId> for Mapper<AccountId, T, C, B, GetCurrencyId>
where
	T: fungibles::Mutate<AccountId, Balance = B>,
	C: ConvertBalance<
		<T as fungibles::Inspect<AccountId>>::Balance,
		B,
		AssetId = <T as fungibles::Inspect<AccountId>>::AssetId,
	>,
	B: BalanceT,
	GetCurrencyId: Get<<T as fungibles::Inspect<AccountId>>::AssetId>,
{
	fn mint_into(dest: &AccountId, amount: Self::Balance) -> DispatchResult {
		T::mint_into(
			GetCurrencyId::get(),
			dest,
			C::convert_balance_back(amount, GetCurrencyId::get())?,
		)
	}

	fn burn_from(dest: &AccountId, amount: Self::Balance) -> Result<Self::Balance, DispatchError> {
		T::burn_from(
			GetCurrencyId::get(),
			dest,
			C::convert_balance_back(amount, GetCurrencyId::get())?,
		)
	}
}
