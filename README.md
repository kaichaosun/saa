# Substrate Account Abstraction

| User cases                                     | Category              | Ethereum Implementation | Substrate Implementation                                                            |
| :--------------------------------------------- | :-------------------- | :---------------------- | :---------------------------------------------------------------------------------- |
| Social recovery                                | recovery              |                         | pallet-recovery                                                                     |
| dead man's switch                              | recovery              |                         | [saa-heir](https://github.com/substrate-onestop/account-abstraction/tree/main/heir) |
| Multisig                                       | signature abstraction |                         | pallet-multisig + MultiSignature                                                    |
| Per-device keys                                | signature abstraction |                         |                                                                                     |
| BLS aggregation                                | signature abstraction |                         |                                                                                     |
| Quantum resistant signatures                   | signature abstraction |                         |                                                                                     |
| Spending limit                                 | roles & policies      |                         | No                                                                                  |
| Multiple roles                                 | roles & policies      |                         | pallet-proxy                                                                        |
| Session keys                                   | roles & policies      |                         |                                                                                     |
| gas sponsorship models                         | gas abstraction       |                         |                                                                                     |
| pay gas with ERC20 tokens                      | gas abstraction       |                         |                                                                                     |
| privacy - without buying ETH                   | gas abstraction       |                         |                                                                                     |
| cross-chain operation                          | gas abstraction       |                         |                                                                                     |
| batching and atomicity                         | batch & automation    |                         | pallet-utility                                                                      |
| automating time-delayed and event-driven flows | batch & automation    |                         |                                                                                     |
|                                                |                       |                         |                                                                                     |
