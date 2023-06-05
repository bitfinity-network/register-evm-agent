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

dfx canister call oracle_canister register_self_in_evmc '(record {r="0xd0d4acd61af63514512cbe84ae70ee08e8bf8d78c0e4ec9bdec8b2acf6f92e8f";s="0x297a0115b08516cd8394a00d5579240935674ebb7ebd155c7fc966dcc3a8ba76";v="0xad675";to=opt "0xb0e5863d0ddf7e105e409fee0ecc0123a362e14b";gas="0";maxFeePerGas=null;gasPrice=null;value="100000";blockNumber=null;from="0xf5d64c6d022ec40224c563e722b80c2312823e5b";hash="0xa8841791fff989c08d2bd4b9cf637db90317b11fa0f743c02cd1f591374a1e7c";blockHash=null;"type"=null;accessList=null;transactionIndex=null;nonce="0";maxPriorityFeePerGas=null;input="";chainId=opt "355113"}, vec{252:nat8;242:nat8;203:nat8;223:nat8;23:nat8;26:nat8;33:nat8;196:nat8;128:nat8;170:nat8;127:nat8;83:nat8;215:nat8;127:nat8;63:nat8;187:nat8;240:nat8;34:nat8;130:nat8;243:nat8;255:nat8;9:nat8;156:nat8;120:nat8;227:nat8;17:nat8;251:nat8;55:nat8;52:nat8;140:nat8;114:nat8;247:nat8;})' --network ic

# if return (variant { Err = variant { Internal = "Account already registered" } })
dfx canister --network ic call oracle_canister reset_self_account 
```