# NFT registry pallet
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Build Status](https://travis-ci.com/juniuszhou/nft-registry.svg?branch=master)](https://travis-ci.com/juniuszhou/nft-registry)
[![codecov](https://codecov.io/gh/juniuszhou/nft-registry/branch/master/graph/badge.svg)](https://codecov.io/gh/juniuszhou/nft-registry)

## Overview
The project is to implement the whole process of mint the private-enable NFT (pNFT).

private-enable means the token minting from anchored document that already verified in p2p network.

## Process of mint a pNFT
1. User put the document in p2p network and then be verified
2. Verified document anchored in substrate network
3. Create contract to implement customized logic to verify proofs against document root
4. Register the contract in NFT pallet
5. Mint the token with registered verification contract
6. NFT palllet call contract to apply costumized logic
7. Contract call back the NFT pallet to complete mint process

## Four components
To realize the whol mint process, the whole project includes four components as following.
### NFT Registry
It is the major component for mint private NFT, and the main entry for extrinsic calls.

### ERC721
ERC721 is designed as a separate deliverable module for two different deployment scenario.
1. ERC721 as a separate module in runtime, expose the interface such mint, transfer, burn.
2. ERC721 works as the foundamental functionality used by other pallet, like NFT pallet.
   
### Validation contract
To mint a pNFT, we need apply some customized logic to verify the merkle proofs against anchored document root.
We put the logic in smart contract, considering the flexibility of contract can be deployed and executed without the runtime upgrade.
We provide a template and framework how to build a contract with Ink!

### Call
After the merkle proofs varified in contract, it will call back to runtime to mint a token. In call component, there are some enums, types needed to be defined, then call runtime from contract is doable.


## Implementation details

### The ERC721 design
There are lots discussion about how we implement the ERC721, if it should be a pallet can be separately delivery or just foundamental implementation as the basic pallet behind the pNFT. We should implement it completly align with the specification of Ethereum, or need adapt according to constraints and advantages of Substrete.
1. The length of index, u64 is enough and Substrate can support it well. 
   Considering some project need migrate data from Ethereum, the length is u256. So we keep the index configurable in Trait.

2. put all real implementation in module, just keep the origin checking in enum call. then other pallet can flexibly integrate with it easily.
   a. set this pallet as a module in runtime, then it can be used as separate pallet.
   b. use it as basic mod behind the other pallet, similar with our usage for nft case. then this pallet uncessarily as a module in runtime. 

3. safe_transfer_from
   In Ethereum, safe transfer is defined for ERC721 to avoid the token transferred to a contract address which not implement IERC721Receiver-onERC721Received.
   but this check can not be done in Substrate. So we don't provide such interface.

4. get all tokens owned by account interface
   We provide the interface as the same with Ethereum. User can get all owned tokens.

5. how to support enumaration in substrate 
   In Ethereum, the index for tokens owned by account and index for all minted are maintained to make the access to each token (via account id and token index) faster. We can iterate all items stored in map via prefix in Substrate. But we still need maintain a continuous token index to efficiently access to token.

### The pNFT design
To complete the mint process, NFT pallet need get document root from anchor pallet. The verification algorithm is defined in the proofs pallet. All information like token id, token owner, token index stored in the ERC721 pallet. NFT just store the data like varification contract, reserved currency and token's metadata.

### Call
The module is mainly dealing with encode enums like pallet module index and method index in pallet, also define the data type such as Hash, AccountId, Blocknunmber and so on. Unit test in the module focus on if the encoded bytes for a method are the same between this module and real runtime node.
Unit test for NFT mint can not be done since the NFT pallet not merged into a node runtime yet. 

### Validation
The module give a template to write a contract to verify proofs based on Ink.

