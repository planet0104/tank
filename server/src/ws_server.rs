extern crate websocket;
extern crate bytes;
extern crate tokio_core;
extern crate hyper;
use std::io;
use std::net::ToSocketAddrs;
use tokio_core::net::{TcpListener, TcpStream};
use futures::{Stream, Future};
use websocket::server::InvalidConnection;
use tokio_core::reactor::Handle;
use websocket::server::async::Incoming as WsIncoming;
use websocket::server::upgrade::async::IntoWs;
use hyper::method::Method;
use hyper::uri::RequestUri;
use hyper::version::HttpVersion;
use websocket::header::Headers;
use hyper::http::h1::Incoming;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use websocket::server::upgrade::async::Upgrade;

pub struct WsServer{
	listener: TcpListener
}

impl WsServer{
	pub fn bind<A: ToSocketAddrs>(addr: A, handle: &Handle) -> io::Result<Self> {
		let tcp = ::std::net::TcpListener::bind(addr)?;
		let address = tcp.local_addr()?;
		Ok(WsServer {
		       listener: TcpListener::from_listener(tcp, &address, handle)?,
		   })
	}

	pub fn incoming(self) -> WsIncoming<TcpStream> {
		let future = self.listener
				.incoming()
				.map_err(|e| {
					InvalidConnection {
						stream: None,
						parsed: None,
						buffer: None,
						error: e.into(),
					}
				}).and_then(|(stream, a)| {
					stream.into_ws()
						.map(move |u| (u, a))
						.or_else(|(stream, _req, buf , err)|{
							println!("无效websocket连接:{:?}", err);
							Ok((Upgrade{
									headers: Headers::new(),
									stream: stream,
									request: Incoming{
										version: HttpVersion::Http09,
										subject: (Method::Get, RequestUri::AbsolutePath(String::from("/"))),
										headers: Headers::new(),
									},
									buffer: buf,
								}, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80)))
						})
				});
		Box::new(future)
	}
}