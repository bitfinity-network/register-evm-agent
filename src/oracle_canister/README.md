# Oracle canister

## build
```sh
cargo run -p oracle_canister --features "export-api" > .artifact/oracle_canister.did

cargo build --target wasm32-unknown-unknown --release --package oracle_canister --features "export-api"

ic-wasm target/wasm32-unknown-unknown/release/oracle_canister.wasm -o .artifact/oracle_canister.wasm shrink
```

## deploy local
terminal 0:
```sh
dfx start --clean
```

terminal 1:
```sh
dfx canister create --no-wallet oracle_canister

dfx build oracle_canister

dfx canister install oracle_canister --argument "record { evmc_principal=principal \"aaaaa-aa\";owner=principal \"$(dfx identity get-principal)\"}"

# add cryptocurrency pairs
dfx canister call oracle_canister add_pair '("bitcoin")'
dfx canister call oracle_canister add_pair '("ethereum")'
dfx canister call oracle_canister add_pair '("internet-computer")'
dfx canister call oracle_canister add_pair '("ordinals")'
dfx canister call oracle_canister add_pair '("dfuk")'
dfx canister call oracle_canister add_pair '("pepebrc")'
dfx canister call oracle_canister add_pair '("pizabrc")'
dfx canister call oracle_canister add_pair '("biso")'
dfx canister call oracle_canister add_pair '("meme-brc-20")'
```

Open link: `http://127.0.0.1:8000/?canisterId=<Oracle_Canister_Id>` such as `http://127.0.0.1:8000/?canisterId=bnz7o-iuaaa-aaaaa-qaaaa-cai` in browser. 

## work with test evmc
```sh
dfx build oracle_canister --network ic

dfx canister install oracle_canister --argument "record { evmc_principal=principal \"4fe7g-7iaaa-aaaak-aegcq-cai\";owner=principal \"$(dfx identity get-principal)\"}" -m=upgrade --network ic


# add supported cryptocurrencies
dfx canister call oracle_canister add_pair '("bitcoin")' --network ic
dfx canister call oracle_canister add_pair '("ethereum")' --network ic
dfx canister call oracle_canister add_pair '("internet-computer")' --network ic
dfx canister call oracle_canister add_pair '("ordinals")' --network ic
dfx canister call oracle_canister add_pair '("dfuk")' --network ic
dfx canister call oracle_canister add_pair '("pepebrc")' --network ic
dfx canister call oracle_canister add_pair '("pizabrc")' --network ic
dfx canister call oracle_canister add_pair '("biso")' --network ic
dfx canister call oracle_canister add_pair '("meme-brc-20")' --network ic

# ... the canister will get the price pair automatically


dfx canister call oracle_canister set_evmc_principal '(principal "4fe7g-7iaaa-aaaak-aegcq-cai")' --network ic

dfx canister call oracle_canister get_evmc_principal --network ic --query
(principal "4fe7g-7iaaa-aaaak-aegcq-cai")

dfx canister call oracle_canister register_self_in_evmc '(record {r="0xe21d09891bd17b7ab593d8d14dec33d31e9ce5e4bb6e8777806a3c7298d62556";s="0x69fb251d1703aa958aaa59307dd50a7cd505f8a64080ae5b5199a130ceaf4060";v="0xad675";to=opt "0xb0e5863d0ddf7e105e409fee0ecc0123a362e14b";gas="0xcf08";maxFeePerGas=null;gasPrice=opt "0x0";value="0x186a0";blockNumber=null;from="0x6bfdb1675a8e3baf666523a77a32faece02e2343";hash="0x9c565e195da2ed050f4dbd0181354c6c38e5db4014b0b686b2fde2531373d702";blockHash=null;"type"=null;accessList=null;transactionIndex=null;nonce="0x0";maxPriorityFeePerGas=null;input="";chainId=opt "355113"}, vec{252:nat8;242:nat8;203:nat8;202:nat8;23:nat8;26:nat8;33:nat8;228:nat8;128:nat8;175:nat8;127:nat8;83:nat8;215:nat8;127:nat8;63:nat8;171:nat8;240:nat8;226:nat8;130:nat8;243:nat8;250:nat8;9:nat8;156:nat8;120:nat8;227:nat8;25:nat8;203:nat8;55:nat8;52:nat8;204:nat8;114:nat8;247:nat8})' --network ic

# if return (variant { Err = variant { Internal = "Account already registered" } })
dfx canister --network ic call oracle_canister reset_self_account 
```