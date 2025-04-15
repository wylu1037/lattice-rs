# WALLET

| 模块              | 功能说明                                  |
|-----------------|---------------------------------------|
| **私钥管理**        | 安全生成/存储/导入私钥（secp256k1 椭圆曲线）          |
| **地址派生**        | 从私钥推导出以太坊地址（Keccak-256 哈希 + 十六进制编码）   |
| **交易签名**        | 对交易数据进行签名（符合 EIP-155/2930/1559 标准）    |
| **网络交互**        | 连接以太坊节点（JSON-RPC 或 WebSocket）         |
| **助记词 & HD 钱包** | 支持 BIP-39（助记词）、BIP-32/BIP-44（分层确定性钱包） |
| **ERC-20 支持**   | 代币转账、余额查询等                            |

### 安全注意事项

+ 私钥存储：使用硬件加密（如 AWS KMS）或操作系统密钥环（如 keyring 库）
+ 依赖审计：定期检查 Cargo.lock 中的依赖漏洞（cargo audit）
+ 环境隔离：测试网与主网分离

### 进阶功能

+ 多链支持：通过 Chain 枚举切换网络
+ Gas 策略：实现 GasOracle 动态调整 Gas 价格
+ 合约交互：使用 ethers-rs 的 Contract 模块

### 开发工具链

+ 部署：编译为 WASM 可嵌入前端（如 Web3 网页钱包）
+ 考虑作为浏览器的扩展使用

---

# 以太坊中的 BIP32、BIP39 和 BIP44

区块链中定义了一套标准，用于生成和管理层次确定性钱包（HD 钱包），其中 BIP32、BIP39 和 BIP44 构成了核心协议。以下是它们的详细说明及应用。

## BIP32: 层次确定性密钥树

BIP32（Bitcoin Improvement Proposal #32）是一个用于生成和管理分层确定性密钥树的协议。

### 特点：

1. **随机种子生成**：BIP32 使用随机种子生成一组可预测的密钥对。通过一个种子（seed），用户可以从一个根密钥（master
   key）派生出多个子密钥，从而避免了管理多个私钥的麻烦[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)[<sup data-citation='{&quot;url&quot;:&quot;https://segmentfault.com/a/1190000017103354&quot;,&quot;title&quot;:&quot;HP钱包概念及底层原理 (BIP32，BIP39，BIP44） - 区块链探索者 - SegmentFault 思否&quot;,&quot;content&quot;:&quot;概念是由BIP32（Bitcoin Improvement Proposals）提出，BIP39解决seed难以抄写记忆问题，BIP44规范各个币种路径规范达成业界共识。 至此修成正果成为分层钱包的集大成者。&quot;}'>2</sup>](https://segmentfault.com/a/1190000017103354)。
2. **层级结构**
   ：密钥的派生是分层的，用户可以为不同的用途（如多个账户或区块链应用）使用不同的密钥，从而增强安全性[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)。

BIP32 为区块链钱包及密钥管理奠定了技术基石，是分层确定性钱包的基础标准。

---

## BIP39: 助记词标准

BIP39
的出现解决了种子（seed）难以记忆和抄写的问题。它通过一种助记词生成方式，将复杂的随机种子转化为一组易读易记的单词列表[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)。

### 特点：

1. **助记词生成**：BIP39 将随机熵转换为一组助记词，这些助记词可以用来生成种子，而该种子是 BIP32
   的入口[<sup data-citation='{&quot;url&quot;:&quot;https://fpchen.readthedocs.io/zh/latest/note/BlockChain/wallet/BIP39-BIP32-BIP44.html&quot;,&quot;title&quot;:&quot;4. BIP39、BIP44、BIP32 协议 — fpchen note V0.1.0 文档&quot;,&quot;content&quot;:&quot;4. BIP39、BIP44、BIP32 协议 ¶ 4.1. HD 钱包导入流程 ¶ 4.2. BIP39 ¶ 熵每次都可以得到不同的助记词&quot;}'>5</sup>](https://fpchen.readthedocs.io/zh/latest/note/BlockChain/wallet/BIP39-BIP32-BIP44.html)。
2. **用户友好**
   ：用户只需记住助记词即可恢复钱包，大大简化了备份过程[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)[<sup data-citation='{&quot;url&quot;:&quot;https://segmentfault.com/a/1190000017103354&quot;,&quot;title&quot;:&quot;HP钱包概念及底层原理 (BIP32，BIP39，BIP44） - 区块链探索者 - SegmentFault 思否&quot;,&quot;content&quot;:&quot;概念是由BIP32（Bitcoin Improvement Proposals）提出，BIP39解决seed难以抄写记忆问题，BIP44规范各个币种路径规范达成业界共识。 至此修成正果成为分层钱包的集大成者。&quot;}'>2</sup>](https://segmentfault.com/a/1190000017103354)。

这种助记词标准已成为区块链行业的通用协议，并被大多数钱包所采用。

---

## BIP44: 多币种路径定义

BIP44 是基于 BIP32
的一种路径定义规范，用来支持多币种账户的分层路径生成[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)[<sup data-citation='{&quot;url&quot;:&quot;https://zhuanlan.zhihu.com/p/293110448&quot;,&quot;title&quot;:&quot;数字钱包 HD Wallet（BIP32密钥派生） - 知乎&quot;,&quot;content&quot;:&quot;BIP43和BIP44统一了钱包软件对分层路径和数字编号的理解和使用，使得了用户用相同的助记词在同一区块链中能够确定性地生成相同的一组密钥。 BIP39、BIP43、BIP44都是兼容BIP32的协议，后来还被比特币以外的区块链所借鉴，形成了区块链行业的共识。&quot;}'>3</sup>](https://zhuanlan.zhihu.com/p/293110448)。

### 特点：

1. **路径规范**：BIP44 定义了多层路径规则，用于标记不同币种、账户和用途。其通用结构为：
    ```
    m / purpose' / coin_type' / account' / change / address_index
    ```
    - `purpose`: 固定为 44，标识为 BIP44 协议。
    - `coin_type`:
      表示币种，比如比特币（0）或以太坊（60）[<sup data-citation='{&quot;url&quot;:&quot;https://zhuanlan.zhihu.com/p/293110448&quot;,&quot;title&quot;:&quot;数字钱包 HD Wallet（BIP32密钥派生） - 知乎&quot;,&quot;content&quot;:&quot;BIP43和BIP44统一了钱包软件对分层路径和数字编号的理解和使用，使得了用户用相同的助记词在同一区块链中能够确定性地生成相同的一组密钥。 BIP39、BIP43、BIP44都是兼容BIP32的协议，后来还被比特币以外的区块链所借鉴，形成了区块链行业的共识。&quot;}'>3</sup>](https://zhuanlan.zhihu.com/p/293110448)。
    - `account`: 支持多账户体系。
    - `change`: 区分主钱包地址和找零地址。
    - `address_index`: 标识具体的地址序号。
2. **行业共识**：BIP44
   最显著的贡献是将多币种路径纳入规范，使得钱包软件能够达成一致，生成确定性的密钥对[<sup data-citation='{&quot;url&quot;:&quot;https://segmentfault.com/a/1190000017103354&quot;,&quot;title&quot;:&quot;HP钱包概念及底层原理 (BIP32，BIP39，BIP44） - 区块链探索者 - SegmentFault 思否&quot;,&quot;content&quot;:&quot;概念是由BIP32（Bitcoin Improvement Proposals）提出，BIP39解决seed难以抄写记忆问题，BIP44规范各个币种路径规范达成业界共识。 至此修成正果成为分层钱包的集大成者。&quot;}'>2</sup>](https://segmentfault.com/a/1190000017103354)[<sup data-citation='{&quot;url&quot;:&quot;https://zhuanlan.zhihu.com/p/293110448&quot;,&quot;title&quot;:&quot;数字钱包 HD Wallet（BIP32密钥派生） - 知乎&quot;,&quot;content&quot;:&quot;BIP43和BIP44统一了钱包软件对分层路径和数字编号的理解和使用，使得了用户用相同的助记词在同一区块链中能够确定性地生成相同的一组密钥。 BIP39、BIP43、BIP44都是兼容BIP32的协议，后来还被比特币以外的区块链所借鉴，形成了区块链行业的共识。&quot;}'>3</sup>](https://zhuanlan.zhihu.com/p/293110448)。

---

## 三者的关系

这三者在 HD 钱包中的作用各有侧重，之间具有衔接关系：

1. **BIP32** 提供分层确定性密钥树的生成和管理方法。
2. **BIP39** 为 BIP32
   提供了用户友好的助记词方案，使得随机种子易于记忆和使用[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)[<sup data-citation='{&quot;url&quot;:&quot;https://segmentfault.com/a/1190000017103354&quot;,&quot;title&quot;:&quot;HP钱包概念及底层原理 (BIP32，BIP39，BIP44） - 区块链探索者 - SegmentFault 思否&quot;,&quot;content&quot;:&quot;概念是由BIP32（Bitcoin Improvement Proposals）提出，BIP39解决seed难以抄写记忆问题，BIP44规范各个币种路径规范达成业界共识。 至此修成正果成为分层钱包的集大成者。&quot;}'>2</sup>](https://segmentfault.com/a/1190000017103354)。
3. **BIP44** 为 BIP32
   定义了路径规则，支持多币种、账户和地址的分层管理[<sup data-citation='{&quot;url&quot;:&quot;https://imtoken.fans/t/topic/390&quot;,&quot;title&quot;:&quot;如何理解钱包涉及的 BIP32、BIP44、BIP39 - 中文区 - imToken Fans&quot;,&quot;content&quot;:&quot;一句话概括下 BIP44 就是：给 BIP32 的分层路径定义规范。 什么是 BIP39？ BIP32 能够让我们保存一个随机数种子，而不是一堆密钥。但是对于大部分用户来讲，还是非常不友好，这就促使了 BIP39 的出现。它使用助记词的方式生成种子，这种情况下用户只要记住助记词，就可以创造出随机种子作为 BIP32 的&quot;}'>1</sup>](https://imtoken.fans/t/topic/390)[<sup data-citation='{&quot;url&quot;:&quot;https://zhuanlan.zhihu.com/p/293110448&quot;,&quot;title&quot;:&quot;数字钱包 HD Wallet（BIP32密钥派生） - 知乎&quot;,&quot;content&quot;:&quot;BIP43和BIP44统一了钱包软件对分层路径和数字编号的理解和使用，使得了用户用相同的助记词在同一区块链中能够确定性地生成相同的一组密钥。 BIP39、BIP43、BIP44都是兼容BIP32的协议，后来还被比特币以外的区块链所借鉴，形成了区块链行业的共识。&quot;}'>3</sup>](https://zhuanlan.zhihu.com/p/293110448)。

---

## 总结

BIP32、BIP39 和 BIP44 是分层确定性钱包的重要协议，分别从密钥生成、用户体验和路径规范三个层面完善了 HD
钱包体系，从而形成了区块链行业的广泛共识[<sup data-citation='{&quot;url&quot;:&quot;https://zhuanlan.zhihu.com/p/293110448&quot;,&quot;title&quot;:&quot;数字钱包 HD Wallet（BIP32密钥派生） - 知乎&quot;,&quot;content&quot;:&quot;BIP43和BIP44统一了钱包软件对分层路径和数字编号的理解和使用，使得了用户用相同的助记词在同一区块链中能够确定性地生成相同的一组密钥。 BIP39、BIP43、BIP44都是兼容BIP32的协议，后来还被比特币以外的区块链所借鉴，形成了区块链行业的共识。&quot;}'>3</sup>](https://zhuanlan.zhihu.com/p/293110448)[<sup data-citation='{&quot;url&quot;:&quot;https://fpchen.readthedocs.io/zh/latest/note/BlockChain/wallet/BIP39-BIP32-BIP44.html&quot;,&quot;title&quot;:&quot;4. BIP39、BIP44、BIP32 协议 — fpchen note V0.1.0 文档&quot;,&quot;content&quot;:&quot;4. BIP39、BIP44、BIP32 协议 ¶ 4.1. HD 钱包导入流程 ¶ 4.2. BIP39 ¶ 熵每次都可以得到不同的助记词&quot;}'>5</sup>](https://fpchen.readthedocs.io/zh/latest/note/BlockChain/wallet/BIP39-BIP32-BIP44.html)。

---

## 引用

1. [imToken - BIP32、BIP39 和 BIP44 简介](https://imtoken.fans/t/topic/390)
2. [SegmentFault - 数字钱包协议进化](https://segmentfault.com/a/1190000017103354)
3. [知乎 - 钱包协议解析](https://zhuanlan.zhihu.com/p/293110448)
4. [知乎 - HD 钱包与 BIP 协议](https://zhuanlan.zhihu.com/p/297118107)
5. [fpChen Docs - BIP 协议综述](https://fpchen.readthedocs.io/zh/latest/note/BlockChain/wallet/BIP39-BIP32-BIP44.html)
