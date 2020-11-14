package mongo

import (
	"context"
	"testing"

	"github.com/joho/godotenv"
)

func TestMongoClientConnection(t *testing.T) {
	if err := godotenv.Load("../.env"); err != nil {
		t.Fatalf("Got error %s, while loading dotenv", err.Error())
	}

	ctx := context.Background()
	ctx, cancel := context.WithTimeout(ctx, mongoTimeout)
	defer cancel()

	client, err := NewMongoClient(ctx)
	if err != nil {
		t.Fatalf("Got %s, while getting the Mongo client", err.Error())
	}

	if err = client.Ping(ctx, nil); err != nil {
		t.Fatalf("Got %s, while sending ping to Mongo database", err.Error())
	}

	if err = client.Disconnect(ctx); err != nil {
		t.Fatalf("Got %s, while disconnecting from the MongoDB", err.Error())
	}
}
