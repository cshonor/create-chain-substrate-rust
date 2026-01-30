#!/bin/bash
# Substrate 节点部署脚本
# 在服务器上执行，用于部署新构建的节点

set -e

DEPLOY_DIR="$HOME/deployments/substrate"
BINARY_NAME="minimal-template-node"
SERVICE_NAME="substrate-node"
BACKUP_DIR="$HOME/deployments/backups"

echo "=========================================="
echo "开始部署 Substrate 节点..."
echo "=========================================="

# 检查二进制文件是否存在
if [ ! -f "$DEPLOY_DIR/$BINARY_NAME" ]; then
    echo "错误: 未找到二进制文件 $DEPLOY_DIR/$BINARY_NAME"
    exit 1
fi

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 备份旧版本（如果存在）
if [ -f "/usr/local/bin/$BINARY_NAME" ]; then
    echo "备份旧版本..."
    BACKUP_FILE="$BACKUP_DIR/${BINARY_NAME}.$(date +%Y%m%d_%H%M%S)"
    cp "/usr/local/bin/$BINARY_NAME" "$BACKUP_FILE"
    echo "已备份到: $BACKUP_FILE"
fi

# 停止服务（如果正在运行）
if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "停止服务..."
    sudo systemctl stop "$SERVICE_NAME"
fi

# 复制新二进制文件
echo "安装新版本..."
sudo cp "$DEPLOY_DIR/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME"
sudo chmod +x "/usr/local/bin/$BINARY_NAME"

# 验证安装
if [ -f "/usr/local/bin/$BINARY_NAME" ]; then
    echo "安装成功！"
    echo "版本信息:"
    /usr/local/bin/$BINARY_NAME --version || echo "无法获取版本信息"
    echo ""
    echo "文件大小:"
    ls -lh "/usr/local/bin/$BINARY_NAME"
else
    echo "错误: 安装失败"
    exit 1
fi

# 重启服务（如果服务存在）
if systemctl list-unit-files | grep -q "$SERVICE_NAME.service"; then
    echo "重启服务..."
    sudo systemctl restart "$SERVICE_NAME"
    sleep 2
    
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        echo "服务运行正常！"
        echo "查看日志: sudo journalctl -u $SERVICE_NAME -f"
    else
        echo "警告: 服务启动失败，请检查日志"
        echo "查看日志: sudo journalctl -u $SERVICE_NAME -n 50"
    fi
else
    echo "提示: 服务未配置，请手动启动节点"
    echo "启动命令: /usr/local/bin/$BINARY_NAME --dev --tmp"
fi

echo "=========================================="
echo "部署完成！"
echo "=========================================="

