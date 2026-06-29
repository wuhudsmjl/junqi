use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::protocol::ServerMessage;

/// 网络客户端
pub struct GameClient {
    /// 消息发送通道
    tx: Sender<ServerMessage>,
    /// 消息接收通道
    rx: Receiver<ServerMessage>,
    /// 是否已连接
    _connected: bool,
}

impl GameClient {
    /// 连接到服务器
    pub fn connect(addr: SocketAddr) -> Result<Self, String> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
            .map_err(|e| format!("连接失败: {}", e))?;
        stream.set_nonblocking(false).map_err(|e| format!("设置阻塞模式失败: {}", e))?;

        let (tx, rx) = Self::spawn_io_threads(stream);
        Ok(GameClient { tx, rx, _connected: true })
    }

    /// 启动读写线程
    fn spawn_io_threads(stream: TcpStream) -> (Sender<ServerMessage>, Receiver<ServerMessage>) {
        let (out_tx, out_rx) = mpsc::channel::<ServerMessage>();
        let (in_tx, in_rx) = mpsc::channel::<ServerMessage>();

        let mut write_stream = stream.try_clone().expect("clone stream");
        let write_tx = out_tx.clone();
        thread::spawn(move || {
            let heartbeat_tx = write_tx.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(10));
                if heartbeat_tx.send(ServerMessage::Ping).is_err() {
                    break;
                }
            });

            while let Ok(msg) = out_rx.recv() {
                let mut line = serde_json::to_string(&msg).unwrap_or_default();
                line.push('\n');
                if write_stream.write_all(line.as_bytes()).is_err() {
                    break;
                }
                if write_stream.flush().is_err() {
                    break;
                }
            }
        });

        thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some(msg) = ServerMessage::from_line(line.trim()) {
                            if in_tx.send(msg).is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        (out_tx, in_rx)
    }

    /// 发送消息
    pub fn send(&self, msg: ServerMessage) -> Result<(), String> {
        self.tx.send(msg).map_err(|e| format!("发送失败: {}", e))
    }

    /// 尝试接收消息（非阻塞）
    pub fn try_recv(&self) -> Option<ServerMessage> {
        self.rx.try_recv().ok()
    }

    /// 接收消息（阻塞，带超时）
    pub fn recv_timeout(&self, timeout: Duration) -> Option<ServerMessage> {
        self.rx.recv_timeout(timeout).ok()
    }
}
