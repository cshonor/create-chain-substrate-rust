# 多阶段构建：构建阶段
# 使用 Substrate 官方 CI 镜像（包含 Rust 1.88.0 和所有依赖）
FROM docker.io/paritytech/ci-unified:latest as builder

# 设置工作目录
WORKDIR /substrate

# 复制项目文件
COPY . /substrate

# 设置构建环境变量
ENV RUSTFLAGS='-A useless_deprecated'
ENV WASM_BUILD_TOOLCHAIN=nightly-2024-07-01
ENV CARGO_BUILD_JOBS=4

# 获取依赖（利用 Docker 层缓存）
RUN cargo fetch --locked

# 构建项目
RUN cargo build --workspace --locked --release

# 运行阶段：最小化镜像
FROM docker.io/parity/base-bin:latest

# 从构建阶段复制二进制文件
COPY --from=builder /substrate/target/release/minimal-template-node /usr/local/bin/

# 创建非 root 用户
USER root
RUN useradd -m -u 1001 -U -s /bin/sh -d /substrate substrate && \
	mkdir -p /data /substrate/.local/share && \
	chown -R substrate:substrate /data && \
	ln -s /data /substrate/.local/share/substrate && \
# 清理不必要的文件，减少攻击面
	rm -rf /usr/bin /usr/sbin && \
# 验证二进制文件可以运行
	/usr/local/bin/minimal-template-node --version

# 切换到非 root 用户
USER substrate

# 暴露端口
# 30333: P2P 端口
# 9933: RPC HTTP 端口
# 9944: RPC WebSocket 端口
# 9615: Prometheus 指标端口
EXPOSE 30333 9933 9944 9615

# 数据卷（链数据持久化）
VOLUME ["/data"]

# 默认入口点
ENTRYPOINT ["/usr/local/bin/minimal-template-node"]

# 默认命令：启动开发模式节点
CMD ["--dev", "--rpc-external", "--ws-external"]
