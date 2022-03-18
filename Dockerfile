FROM rust

ADD solutions.csv .
ADD words.csv .
ADD freq_map.json .
ADD Cargo.toml Cargo.toml
ADD Cargo.lock Cargo.lock
ADD src src

RUN cargo build --release

CMD ["./target/release/wordle-squire"]  
