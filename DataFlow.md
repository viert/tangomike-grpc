### Proposed dataflow

_In order to authenticate a token must be issued via TangoMike API. The token then must be set in the client._

#### Create a new flight

- Client requests TangoMike Create Flight API handler with the pre-configured token in the `Authorization` header.
- If the token is correct, the user is authenticated and a new flight is being created.

Basic flight model structure

```python
class Flight:
  flight_id: str  # a unique id (uuid4)
  user_id: str  # a reference to the flight owner
  # ... other flight-related fields
```

- When the flight is created it's connected to the authenticated user via its `user_id` field. The `flight_id` field is generated automatically. Both fields are returned back to client.

- Client connects to TangoMike GRPC using `UploadTrackStream` rpc call and sends `flight_id` and `auth_token` fields in the call metadata.
- TangoMike GRPC requests the API on behalf of the user using the auth token to authenticate and checks if the flight_id given belongs to the user.
- If any of the checks fail GRPC service responds with NotFound status.
- The client is free to start sending track entries right after connect. If it receives a NotFound GRPC status at any point it must stop and try to recreate the flight.
