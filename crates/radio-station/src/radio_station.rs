use std::net::{SocketAddr};
use std::time::{Duration, Instant};

use futures::future::{self, Either};
use futures::sink::SinkExt;
use futures::stream::{SplitSink, SplitStream, StreamExt as FutStreamExt};
use ogg::reading::PacketReader;
use srs::message::Position;
use srs::{Client, VoiceStream};
use tokio::time::delay_for;
use std::io;
use crate::input;

pub struct RadioStation {
    name: String,
    position: Position,
    freq: u64,
    port: u16,
}

impl RadioStation {
    pub fn new(name: &str) -> Self {
        RadioStation {
            name: name.to_string(),
            position: Position::default(),
            freq: 251_000_000,
            port: 5002,
        }
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn set_position(&mut self, x: f64, y: f64, alt: f64) {
        self.position = Position { x, y, alt };
    }

    pub fn set_frequency(&mut self, freq: u64) {
        self.freq = freq;
    }

    pub async fn play<>(
        self,
        server: &str
    ) -> Result<(), anyhow::Error> {
        let mut client = Client::new(&self.name, self.freq);
        client.set_position(self.position);

        let addr : SocketAddr = server.parse().expect("Invalid hostname:port");
        let (sink, stream) = client.start(addr, None).await?.split();

        let rx = Box::pin(recv_voice_packets(stream));
        let tx = Box::pin(radio_broadcast_stdin(sink));

        match future::try_select(rx, tx).await {
            Err(Either::Left((err, _))) => Err(err.into()),
            Err(Either::Right((err, _))) => Err(err.into()),
            _ => Ok(()),
        }
    }
}

async fn recv_voice_packets(mut stream: SplitStream<VoiceStream>) -> Result<(), anyhow::Error> {
    while let Some(packet) = stream.next().await {
        packet?;
        // we are currently not interested in the received voice packets, so simply discard them
    }

    Ok(())
}

async fn radio_broadcast_stdin<>(
    mut sink: SplitSink<VoiceStream, Vec<u8>>
) -> Result<(), anyhow::Error> {
    loop {
        debug!("Playing STDIN");

        let input = input::Input::Stdin(io::stdin());

        let start = Instant::now();
        let mut audio = PacketReader::new(input);
        let mut frame_count = 0;

        while let Some(pck) = audio.read_packet()? {
            if pck.data.len() == 0 {
                continue;
            }

            sink.send(pck.data).await?;
            frame_count += 1;

            // wait for the current ~playtime before sending the next package
            let playtime = Duration::from_millis((frame_count as u64 + 1) * 20); // 20m per frame count
            let elapsed = start.elapsed();
            if playtime > elapsed {
                delay_for(playtime - elapsed).await;
            }
        }
    }
}
