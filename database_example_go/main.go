package main

import (
	"flag"
	"log"

	"github.com/gomodule/redigo/redis"
)

func newPool() *redis.Pool {
	return &redis.Pool{
		MaxIdle:   5,
		MaxActive: 25,
		Dial: func() (redis.Conn, error) {
			c, err := redis.Dial("tcp", ":6379", redis.DialDatabase(0))
			if err != nil {
				panic(err.Error())
			}
			return c, err
		},
	}

}

var pool = newPool()
var channel = flag.String("c", "chat:*", "Channels to connect to")

func main() {
	flag.Parse()

	c := pool.Get()
	psc := redis.PubSubConn{Conn: c}
	psc.PSubscribe(*channel)
	log.Printf("subscribed to: %s", *channel)
	for {
		switch v := psc.Receive().(type) {
		case redis.Message:
			log.Printf("recv from %s: %s", v.Channel, v.Data)
		case error:
			log.Printf("%v", v)
		}
	}
}
