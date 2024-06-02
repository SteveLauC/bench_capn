A toy Cap'n Proto QPS benchmark tool.


## NOTE

   1. This tool will run a ping-pong test, the message is just a string, i.e.,
      no special serialization/deserialization cost.

   2. Server and client, they are all single-threaded.

   3. If you run this benchmark on Linux, then server thread will be bound to 
      core 0, and client will use core 1.

## Run the benchmark

1. Build the release binaries
   
   ```sh
   $ cargo b -r
   ```

2. Start the server

   ```sh
   ./target/release/server 127.0.0.1:11451
   INFO: Listening on 127.0.0.1:9100
   INFO: Bound to core 0
   ```

3. Start the client

   ```sh
   $ ./target/release/client 127.0.0.1:9100 1 2
   INFO: Benchmark against: 127.0.0.1:9100
   INFO: The # of connections: 1
   INFO: Duration: 2s
   INFO: Bound to core 1
   QPS: 19927
   ```

## Results on my machine

```sh
$ uname -a
Linux fedora 6.2.9-300.fc38.x86_64 #1 SMP PREEMPT_DYNAMIC Thu Mar 30 22:32:58 UTC 2023 x86_64 GNU/Linux
```

| # of connections | QPS |
|------------------|-----|
| 1                |24958|
| 2                |45197|
| 3                |43875|
| 4                |47533|
| 5                |48359|
| 6                |50201|
| 7                |50005|
| 8                |51000|
| 9                |51746|
| 10               |51256|
| 20               |52780|
| 30               |52485|
| 40               |52347|
| 50               |52598|
| 60               |51354|
| 70               |52748|


