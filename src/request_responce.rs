
use async_std::io::{self};
use async_trait::async_trait;
use futures::prelude::*;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed, ProtocolName};
use libp2p::futures::{AsyncRead, AsyncWrite};
use libp2p::request_response::RequestResponseCodec;

use crate::MTU;

#[derive(Clone, Default)]
pub struct PacketStreamProtocol ;

impl ProtocolName for PacketStreamProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/p2p-vpn/0.0.0".as_bytes()
    }
}

pub struct PacketRequest([u8; MTU]);

#[derive(Debug, Clone)]
pub struct PacketStreamCodec;

#[async_trait]
impl RequestResponseCodec for PacketStreamCodec {
    type Protocol = PacketStreamProtocol;
    type Request = Vec<u8>;
    type Response = ();

    async fn read_request<T>(
        &mut self,
        _: &PacketStreamProtocol,
        io: &mut T
    ) -> io::Result<Self::Request>
        where 
            T: AsyncRead + Unpin + Send,
        {
            let vec = read_length_prefixed(io, MTU).await?;
            if vec.is_empty() {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            Ok(vec)
        }

    async fn write_request<T> (
        &mut self, 
        _: &PacketStreamProtocol,
        io: &mut T,
        request: Self::Request,
    ) -> io::Result<()> 
        where T: AsyncWrite + Unpin + Send,
        {
            write_length_prefixed(io, request).await?;
            io.close().await?;
            Ok(())
    }

    async fn read_response<T>(
        &mut self, 
        _: &PacketStreamProtocol,
        _: &mut T
    ) -> io::Result<Self::Response> 
        where T :AsyncRead + Unpin + Send {
            Ok(())
    }

    async fn write_response<T>( 
        &mut self,
        _: &PacketStreamProtocol,
        _: &mut T,
        _: (),
        ) -> io::Result<()> 
        where 
            T: AsyncWrite + Unpin + Send {
            Ok(())
    }
}