# OneBlock+ 进阶课程作业

## 第一节课作业

### 作业 1

编写存证模块的单元测试代码，包括创建存证的测试用例
撤销存证的测试用例
转移存证的测试用例

### 答案：

[pallets/poe/src/tests.rs](./pallets/poe/src/tests.rs)

### 截图：

![tests](./img/lesson01_01.png)

## 第二节课作业

### 作业 1

跟着视频,完成 Kitties 的开发

- Pallet 编译成功
- 单元测试通过
- 加入 kitties pallet 到 runtime 中,node 可以编译通过
- node 节点可以正常启动

### 答案：

[pallets/kitties](./pallets/kitties)

### 截图：

![tests](./img/lesson02_01.png)
![tests](./img/lesson02_02.png)

## 第三节课作业

### 作业 1

跟着视频,完成 Kitties pallet

- 增加 Currency sale/buy 方法后 Pallet 可以编译
- 增加新的测试用例
- 修改 runtime,node 可以编译通过
- node 节点可以启动

### 答案：

[pallets/kitties](./pallets/kitties)

### 截图：

![tests](./img/lesson03_01.png)
![tests](./img/lesson03_02.png)
![tests](./img/lesson03_03.png)

### 作业 2

Runtime 升级

- Kitties palletv2，将 kitties name 扩充到 8 个字节
- 完成 migration 代码
- 验证从 vO-v2,v1-v2 的升级路径

### 答案：

[pallets/kitties](./pallets/kitties)

### 截图：

![tests](./img/lesson03_04.png)

## 第四节课作业

### 作业

- 在 Offchain Worker 中，使用 Offchain Indexing 特性实现从链上向 Offchain Storage 中写入数据
- 使用 js sdk 从浏览器 frontend 获取到前面写入 Offchain Storage 的数据
- 设计一个场景实例（比如获取一个外部的价格信息），实现从 OCW 中向链上发起带签名负载的不签名交易，并在 Runtime 中正确处理

### 答案：

后端代码 [pallets/binance](./pallets/binance)

前端代码 [front_end](./front_end)

### 截图：

![tests](./img/lesson04_01.png)

![tests](./img/lesson04_02.png)

![tests](./img/lesson04_03.png)

![OneBlock+](./img/substrate.png)
