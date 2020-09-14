rm -rf /tmp/libra
./config-builder validator -a "/ip4/0.0.0.0/tcp/40002" -b "/ip4/0.0.0.0/tcp/40002" -d /tmp/libra/0  -i 0 -l "/ip4/0.0.0.0/tcp/40002" -n 1 -o /tmp/libra/0 -s 466c8d398da5292daaa29f028ee300f5b0458a7faa16792e3b5cf16caf5a58a7
./config-builder faucet -o /tmp/libra -s 466c8d398da5292daaa29f028ee300f5b0458a7faa16792e3b5cf16caf5a58a7 -n 1
./libra-node -f /tmp/libra/0/node.yaml -d
