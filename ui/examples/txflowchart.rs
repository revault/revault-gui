use iced::{Align, Column, Element, Length, Sandbox, Settings, Text};

use bitcoin::{consensus::encode, hashes::hex::FromHex, Network, Transaction};
use revault_ui::chart::{FlowChart, FlowChartMessage as Message, Txid};

pub fn transaction_from_hex(hex: &str) -> Transaction {
    let bytes = Vec::from_hex(&hex).unwrap();
    encode::deserialize::<Transaction>(&bytes).unwrap()
}

pub fn main() -> iced::Result {
    Example::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Example {
    flowchart: FlowChart,
    selected_tx: Option<Txid>,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        let deposit1 = transaction_from_hex("02000000000101a0633c79ade54261e15c492c74df1ff8c80950c2d43e0c04b67f9e1837202d4d0000000000feffffff02a019f8070000000022002024c9386efc4c8adef217b2931caed1cf4891f49770526ad5703d6893b49102ce4762894200000000160014a46574ebd02cd775611cd667167c7c0e0389c75602473044022059387b04513edb9e453b73b6bc786d8f3c3f69635825cca56466791cb8eb36ee02207b693ef36162fc0f65ffbc80803a7e64d3a9330953d1fcbd4af76cbe0dfea1fd0121033a62660f6bcb07fce54464b1af49bc6ab42dd62982aeaf141b63665e88cdff6479010000");
        let unvault1 = transaction_from_hex("02000000000101999ca43ceffdfbe623ea6db115b895051a351bdce4313fd3704b8b63a0ae3ba40000000000fdffffff025892f707000000002200206c1c9b72e836a6dec99014406cf88b4df54da9c57b214b99464b987b447a05723075000000000000220020362dcc982d454a66b2e8febd8740769ce23e2030dda9d7f35c08444f8775ab8f040047304402207e93c1fa8a73b155ee918fb82c745cf6ed4a2c58f6c92d7854349584c67a766702200e75f1a301f8338e0ec7082ad2be61d9c5418b7b0b8bf452ff5b3ccd30dc1c2501483045022100cc874a1bfe25e4b2724aa97994d824c181e847015bdb9f4664a5444337988fb1022079801835a0b5c4aa6dd30860f7b6d46fb8fdee032f82dbf9ff6e72cb20fe9a7b0147522103614f52e6ff874bed71cdadeb0ca7bedace71b40f3e8045982c8222fcf2e444fb2102becc5ebd0649c836922ce8a72b54cf9dbdc9d7de56163dcf706368b3b5ade30952ae00000000");
        let deposit2 = transaction_from_hex("02000000000101999ca43ceffdfbe623ea6db115b895051a351bdce4313fd3704b8b63a0ae3ba40100000000feffffff02ce1d663e000000001600145609eb575ae466c49dbbabf917555d3936a8d6ece0432304000000002200200071401c8209b9cddce78d4a7f0447ac5ae7ff1aaaf0b23c47d039b515602aed0247304402200779b079efa05fc6ead8bd7425d2385d37efc7edf74c6f79db39890248f8e3b1022066a27f2e65220d6f04c7fc52124dc4b649ce6299df6ff159e49b48a599cc2ea00121029078365e60b84940862d480210b669fa3c1dafbe518e9ff395320746366fb5e8a4010000");
        let unvault2 = transaction_from_hex("02000000000101a291d1790dabcf8dbea4582616538225ba65ab11a555208b6de8d3659e063ec00100000000fdffffff0298bc220400000000220020c8874f9efdb6d53ee465a21e72d2b86a01aadc37f043286e3b539d6365b0a4d330750000000000002200200d70a091ea22dc0db14ff09c2f5a7504f7ea73bde8cdff194f1cea7687dc28800400483045022100923109e96efbb249848fe70210dd917eedc74c960cd02a98a61656be88fe0474022037e895712c94a29d5c43dfbd54cf0c56682e8288c3cad0b6074ea854fd3c1d21014830450221009dcee6119bbdcbbe513b5bde6a3d960f7763c777f50f790f330c71e8a8891c4b02205a589e738459c757b908749ec6cf0c492814e4d0e6b0a030cd7272096bcd0a1e0147522102f6df74980f0df6e6de298f00798ce53df31594709bc95014bb0f9374d106b57f2102aa12b2ebe1e812fda873df2739d78bff8760814370acb15cad2fded3012490f052ae00000000");
        let spend = transaction_from_hex("02000000000102b82f4bda5f7f2aee012cbdfe24258e4e3117caad6dc6e0c90ceacd220bcee5b200000000000a00000022f340512074d5e5c8874c042e09f7e788a07022aedce2fa00b5ecd1a39b15ec00000000000a00000006a0770000000000002200200d70a091ea22dc0db14ff09c2f5a7504f7ea73bde8cdff194f1cea7687dc2880c0c10d020000000016001439fd6460bc4876e4b8db91b003708532ada21c7390e7850200000000160014532b344ad1ea3397c64f57606d998c61f26cb3edc0de170300000000160014967309eee79b9be232209395e267dbc9357dc29ee8a9080100000000160014dc8efc8b6fb5192855eb2449e120858da5f8f6205b7a6503000000002200200071401c8209b9cddce78d4a7f0447ac5ae7ff1aaaf0b23c47d039b515602aed0300483045022100e309c0d67eeddc5a6740f7cc8dd679696cb213e3ae5684562bf04ae6ce9256ff02206369fcb1e97254d6e71775c5ce73185908fd0667129860f953dedfad50e0a4740183512102a6db8d9cdb53da7175ae9df456499c24ed569b96918f8bb560f0343833d70f092102fe8334ab977d4f6cd14ba25f1dc5153954329d4a68a050e59362884d1141b70552ae6476a914270cbdbbc948dbd56f28a6265c86b31153bacc5388ac6b76a914959beb32358ccd19fb93e9fd25542d0c8d9a2f7c88ac6c935287675ab2680300483045022100ec38954abe5ceedd3c60a7d7ced1e73217f2f7a66fd2efe776906f1d5986804c02203ddf330ecabe2119d4ab6ae573f7b6850c1c6f57c16d24b4c8a2caff68b2197f0183512103f9af2d4e21bc6e1e30f7974146fd728f0dcfa280521172aee1afba659a7290662103b9ad21981e42c41f38df5f8f3ddb80a597d0d3df230818203a8f50e6be443a2652ae6476a914b5e60d45ce94c86e4acace579dbab85fccf05b7288ac6b76a914f6fbf60a5629321aa8dfe9f7d7ce41447ccc670388ac6c935287675ab26800000000");
        Self {
            flowchart: FlowChart::new(
                Network::Regtest,
                vec![deposit1, unvault1, deposit2, unvault2, spend],
            ),
            selected_tx: None,
        }
    }

    fn title(&self) -> String {
        String::from("FlowChart tool - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::TxSelected(selected) => self.selected_tx = selected,
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Text::new(
                    self.selected_tx
                        .map(|id| id.to_string())
                        .unwrap_or("nothing".to_string()),
                )
                .width(Length::Shrink)
                .size(50),
            )
            .push(self.flowchart.view())
            .into()
    }
}