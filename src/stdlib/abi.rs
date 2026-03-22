#[cfg(test)]
pub(crate) const SIG_ERROR_STRING: &str = "Error(string)";
#[cfg(test)]
pub(crate) const SIG_PANIC_UINT256: &str = "Panic(uint256)";

#[cfg(test)]
pub(crate) const SIG_ERC20_BALANCE_OF: &str = "balanceOf(address)";
#[cfg(test)]
pub(crate) const SIG_ERC20_TRANSFER: &str = "transfer(address,uint256)";
#[cfg(test)]
pub(crate) const SIG_ERC20_APPROVE: &str = "approve(address,uint256)";
#[cfg(test)]
pub(crate) const SIG_ERC20_TRANSFER_FROM: &str = "transferFrom(address,address,uint256)";
#[cfg(test)]
pub(crate) const SIG_ERC20_ALLOWANCE: &str = "allowance(address,address)";
#[cfg(test)]
pub(crate) const SIG_ERC20_TOTAL_SUPPLY: &str = "totalSupply()";
#[cfg(test)]
pub(crate) const SIG_ERC20_DECIMALS: &str = "decimals()";
#[cfg(test)]
pub(crate) const SIG_ERC721_OWNER_OF: &str = "ownerOf(uint256)";
#[cfg(test)]
pub(crate) const SIG_ERC721_TOKEN_URI: &str = "tokenURI(uint256)";

#[cfg(test)]
pub(crate) const SIG_ERC20_INSUFFICIENT_BALANCE: &str =
    "ERC20InsufficientBalance(address,uint256,uint256)";
#[cfg(test)]
pub(crate) const SIG_ERC20_INVALID_SENDER: &str = "ERC20InvalidSender(address)";
#[cfg(test)]
pub(crate) const SIG_ERC20_INVALID_RECEIVER: &str = "ERC20InvalidReceiver(address)";

#[cfg(test)]
pub(crate) const SELECTOR_ERROR_STRING: &str = "08c379a0";
#[cfg(test)]
pub(crate) const SELECTOR_PANIC_UINT256: &str = "4e487b71";

pub(crate) const SELECTOR_ERC20_BALANCE_OF: &str = "70a08231";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SELECTOR_ERC20_TRANSFER: &str = "a9059cbb";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SELECTOR_ERC20_APPROVE: &str = "095ea7b3";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SELECTOR_ERC20_TRANSFER_FROM: &str = "23b872dd";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SELECTOR_ERC20_ALLOWANCE: &str = "dd62ed3e";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SELECTOR_ERC20_TOTAL_SUPPLY: &str = "18160ddd";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const SELECTOR_ERC20_DECIMALS: &str = "313ce567";
pub(crate) const SELECTOR_ERC721_OWNER_OF: &str = "6352211e";
pub(crate) const SELECTOR_ERC721_TOKEN_URI: &str = "c87b56dd";

#[cfg(test)]
pub(crate) const SELECTOR_ERC20_INSUFFICIENT_BALANCE: &str = "e450d38c";
#[cfg(test)]
pub(crate) const SELECTOR_ERC20_INVALID_SENDER: &str = "96c6fd1e";
#[cfg(test)]
pub(crate) const SELECTOR_ERC20_INVALID_RECEIVER: &str = "ec442f05";

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn selector_from_signature(signature: &str) -> String {
    use sha3::{Digest, Keccak256};

    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..4])
}

#[cfg(test)]
pub(crate) fn canonical_selector_catalog() -> std::collections::HashMap<&'static str, &'static str>
{
    std::collections::HashMap::from([
        (SIG_ERROR_STRING, SELECTOR_ERROR_STRING),
        (SIG_PANIC_UINT256, SELECTOR_PANIC_UINT256),
        ("ERC20.balanceOf(address)", SELECTOR_ERC20_BALANCE_OF),
        ("ERC20.transfer(address,uint256)", SELECTOR_ERC20_TRANSFER),
        ("ERC20.approve(address,uint256)", SELECTOR_ERC20_APPROVE),
        (
            "ERC20.transferFrom(address,address,uint256)",
            SELECTOR_ERC20_TRANSFER_FROM,
        ),
        ("ERC20.allowance(address,address)", SELECTOR_ERC20_ALLOWANCE),
        ("ERC20.totalSupply()", SELECTOR_ERC20_TOTAL_SUPPLY),
        ("ERC20.decimals()", SELECTOR_ERC20_DECIMALS),
        ("ERC721.ownerOf(uint256)", SELECTOR_ERC721_OWNER_OF),
        ("ERC721.tokenURI(uint256)", SELECTOR_ERC721_TOKEN_URI),
        (
            SIG_ERC20_INSUFFICIENT_BALANCE,
            SELECTOR_ERC20_INSUFFICIENT_BALANCE,
        ),
        (SIG_ERC20_INVALID_SENDER, SELECTOR_ERC20_INVALID_SENDER),
        (SIG_ERC20_INVALID_RECEIVER, SELECTOR_ERC20_INVALID_RECEIVER),
    ])
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_selector_catalog, selector_from_signature, SELECTOR_ERC20_ALLOWANCE,
        SELECTOR_ERC20_APPROVE, SELECTOR_ERC20_BALANCE_OF, SELECTOR_ERC20_DECIMALS,
        SELECTOR_ERC20_INSUFFICIENT_BALANCE, SELECTOR_ERC20_INVALID_RECEIVER,
        SELECTOR_ERC20_INVALID_SENDER, SELECTOR_ERC20_TOTAL_SUPPLY, SELECTOR_ERC20_TRANSFER,
        SELECTOR_ERC20_TRANSFER_FROM, SELECTOR_ERC721_OWNER_OF, SELECTOR_ERC721_TOKEN_URI,
        SELECTOR_ERROR_STRING, SELECTOR_PANIC_UINT256, SIG_ERC20_ALLOWANCE, SIG_ERC20_APPROVE,
        SIG_ERC20_BALANCE_OF, SIG_ERC20_DECIMALS, SIG_ERC20_INSUFFICIENT_BALANCE,
        SIG_ERC20_INVALID_RECEIVER, SIG_ERC20_INVALID_SENDER, SIG_ERC20_TOTAL_SUPPLY,
        SIG_ERC20_TRANSFER, SIG_ERC20_TRANSFER_FROM, SIG_ERC721_OWNER_OF, SIG_ERC721_TOKEN_URI,
        SIG_ERROR_STRING, SIG_PANIC_UINT256,
    };

    #[test]
    fn selector_from_signature_matches_known_vectors() {
        assert_eq!(
            selector_from_signature("transfer(address,uint256)"),
            "a9059cbb"
        );
        assert_eq!(
            selector_from_signature("approve(address,uint256)"),
            "095ea7b3"
        );
        assert_eq!(
            selector_from_signature("ERC20InvalidReceiver(address)"),
            "ec442f05"
        );
    }

    #[test]
    fn selector_is_lowercase_8_hex_chars() {
        let selector = selector_from_signature("balanceOf(address)");
        assert_eq!(selector.len(), 8);
        assert!(selector.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(selector, selector.to_ascii_lowercase());
    }

    #[test]
    fn typed_selector_constants_match_keccak_derivation() {
        assert_eq!(
            selector_from_signature(SIG_ERROR_STRING),
            SELECTOR_ERROR_STRING
        );
        assert_eq!(
            selector_from_signature(SIG_PANIC_UINT256),
            SELECTOR_PANIC_UINT256
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_BALANCE_OF),
            SELECTOR_ERC20_BALANCE_OF
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_TRANSFER),
            SELECTOR_ERC20_TRANSFER
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_APPROVE),
            SELECTOR_ERC20_APPROVE
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_TRANSFER_FROM),
            SELECTOR_ERC20_TRANSFER_FROM
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_ALLOWANCE),
            SELECTOR_ERC20_ALLOWANCE
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_TOTAL_SUPPLY),
            SELECTOR_ERC20_TOTAL_SUPPLY
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_DECIMALS),
            SELECTOR_ERC20_DECIMALS
        );
        assert_eq!(
            selector_from_signature(SIG_ERC721_OWNER_OF),
            SELECTOR_ERC721_OWNER_OF
        );
        assert_eq!(
            selector_from_signature(SIG_ERC721_TOKEN_URI),
            SELECTOR_ERC721_TOKEN_URI
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_INSUFFICIENT_BALANCE),
            SELECTOR_ERC20_INSUFFICIENT_BALANCE
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_INVALID_SENDER),
            SELECTOR_ERC20_INVALID_SENDER
        );
        assert_eq!(
            selector_from_signature(SIG_ERC20_INVALID_RECEIVER),
            SELECTOR_ERC20_INVALID_RECEIVER
        );
    }

    #[test]
    fn canonical_selector_catalog_has_unique_values() {
        let catalog = canonical_selector_catalog();
        let mut unique = std::collections::HashSet::new();
        for selector in catalog.values() {
            assert_eq!(selector.len(), 8);
            assert!(selector.chars().all(|c| c.is_ascii_hexdigit()));
            assert_eq!(*selector, selector.to_ascii_lowercase());
            assert!(
                unique.insert(*selector),
                "duplicate selector in canonical catalog: {}",
                selector
            );
        }
        assert_eq!(unique.len(), catalog.len());
    }
}
