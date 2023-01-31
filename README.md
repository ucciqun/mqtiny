# mqtiny
This project provides MQTiny softwares.
## Usage
### Broker
```
Usage: test [OPTIONS]

Options:
  -p, --port <PORT>  MQTiny service port [default: 1883]
  -h, --help         Print help information
```
example
```
cargo run --bin test -- -p 7001
```
### Publisher
```
cargo run --release -q --bin pub -- --help
Usage: pub [OPTIONS] --ip <IP> --topic <TOPIC>

Options:
  -i, --ip <IP>                            Address of the MQTiny server to connect
  -p, --port <PORT>                        MQTiny service port [default: 1883]
  -c, --count <COUNT>                      Total number of clients [default: 200]
  -I, --interval-of-msg <INTERVAL_OF_MSG>  Interval to publish a message [default: 1000]
  -t, --topic <TOPIC>                      Published topics
  -s, --size <SIZE>                        Message Payload size (bytes) [default: 10]
  -m, --messages <MESSAGES>                Number of messages to publish [default: 5000]
  -q, --qos <QOS>                          QoS level [default: 0]
  -h, --help                               Print help information
```
example
```
cargo run --bin pub -- -i 192.168.0.202 -p 7001 -c 1 -I 1 -t 1 -s 60 -m 10000 -q 0
```
### Subscriber
```
Usage: sub [OPTIONS] --ip <IP> --topic <TOPIC>

Options:
  -i, --ip <IP>        Address of the MQTiny server to connect
  -p, --port <PORT>    MQTiny service port [default: 1883]
  -t, --topic <TOPIC>  Subscrived topics
  -f, --fpga           is FPGA?
  -h, --help           Print help information
```
if the broker is running on the nic-toe, please enable --fpga flag!! 

example
```
cargo run --bin sub -- -i 192.168.0.202 -p 7001 -t 1 --fpga
```
