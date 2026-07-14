# ============================================================================
# 构建阶段：编译 Rust 项目
# ============================================================================
FROM docker.m.daocloud.io/library/rust:1.88 AS builder

# 设置工作目录
WORKDIR /app

# 复制项目文件
COPY . .

# 编译 Release 版本
RUN cargo build --release --package server

# ============================================================================
# 运行阶段：使用精简的镜像
# ============================================================================
FROM docker.m.daocloud.io/library/debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# 创建工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/imazing-server /app/

# 复制配置文件和模板
COPY config /app/config
COPY template.imazingapp /app/template.imazingapp

# 设置环境变量
ENV USE_REAL_SERVICE=true
ENV CONFIG_DIR=/app/config
ENV TEMPLATE_PATH=/app/template.imazingapp
ENV RUST_LOG=info

# 暴露端口（微信云托管默认使用 80 端口）
EXPOSE 80

# 启动应用
CMD ["/app/imazing-server"]
