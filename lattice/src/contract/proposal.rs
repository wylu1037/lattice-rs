use crate::builder::TransactionBuilder;
use crate::impl_builtin_contract;

/// 内置的投票合约
const PROPOSAL_ABI_DEFINITION: &str = r#"[
    {
        "inputs": [
            {
                "internalType": "string",
                "name": "ProposalId",
                "type": "string"
            },
            {
                "internalType": "uint8",
                "name": "VoteSuggestion",
                "type": "uint8"
            }
        ],
        "name": "vote",
        "outputs": [
            {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
            }
        ],
        "stateMutability": "pure",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "string",
                "name": "ProposalId",
                "type": "string"
            }
        ],
        "name": "refresh",
        "outputs": [
            {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
            }
        ],
        "stateMutability": "pure",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "string",
                "name": "proposalId",
                "type": "string"
            }
        ],
        "name": "cancel",
        "outputs": [
            {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
            }
        ],
        "stateMutability": "pure",
        "type": "function"
    }
]"#;

const PROPOSAL_CONTRACT_ADDRESS: &str = "zltc_amgWuhifLRUoZc3GSbv9wUUz6YUfTuWy5";

impl_builtin_contract!(ProposalBuiltinContract, PROPOSAL_ABI_DEFINITION, PROPOSAL_CONTRACT_ADDRESS);

impl ProposalBuiltinContract {
    /// # 投票
    ///
    /// ## 入参
    /// + `proposal_id: &str`: 提案ID
    /// + `approve: bool`: false:反对票、true:同意票
    ///
    /// ## 出参
    /// + `String`: encoded data
    pub fn vote(&self, proposal_id: &str, approve: bool) -> String {
        let approve: String = if approve { String::from("1") } else { String::from("0") };
        let data = self.encode_args("vote", vec![Box::new(proposal_id.to_string()), Box::new(approve)]);
        data
    }

    /// # 取消提案
    ///
    /// ## 入参
    /// + `proposal_id: &str`: 提案ID
    ///
    /// ## 出参
    /// + `String`: encoded data
    pub fn cancel(&self, proposal_id: &str) -> String {
        let data = self.encode_args("cancel", vec![Box::new(proposal_id.to_string())]);
        data
    }

    /// # 刷新提案状态
    ///
    /// ## 入参
    /// + `proposal_id: &str`: 提案ID
    ///
    /// ## 出参
    /// + `String`: encoded data
    pub fn refresh(&self, proposal_id: &str) -> String {
        let data = self.encode_args("refresh", vec![Box::new(proposal_id.to_string())]);
        data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const PROPOSAL_ID: &str = "0x012629af43a2e7cf024cdaeb8c108078b3b62a9f171300000000000000";

    #[test]
    fn test_encode_vote() {
        let data = ProposalBuiltinContract::new().vote(PROPOSAL_ID, true);
        let expect_data = "0x90ca27f300000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000003c30783031323632396166343361326537636630323463646165623863313038303738623362363261396631373133303030303030303030303030303000000000";
        assert_eq!(data, expect_data)
    }

    #[test]
    fn test_encode_cancel() {
        let data = ProposalBuiltinContract::new().cancel(PROPOSAL_ID);
        let expect_data = "0x0b4f3f3d0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003c30783031323632396166343361326537636630323463646165623863313038303738623362363261396631373133303030303030303030303030303000000000";
        assert_eq!(data, expect_data);
    }

    #[test]
    fn test_encode_refresh() {
        let data = ProposalBuiltinContract::new().refresh(PROPOSAL_ID);
        let expect_data = "0x6de8a6090000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003c30783031323632396166343361326537636630323463646165623863313038303738623362363261396631373133303030303030303030303030303000000000";
        assert_eq!(data, expect_data);
    }
}