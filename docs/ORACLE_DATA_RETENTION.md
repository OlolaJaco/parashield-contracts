# Oracle data retention

OracleVerifier retains at most 100 readings for each `(data_type, key)` pair,
for both plaintext and encrypted submissions. A submission replaces the
existing reading from that oracle. When the cap is reached, the oldest stored
entry is evicted to make room for the new reading.

On the next write for a key, stale readings and readings from inactive or
unregistered oracles are pruned. Plaintext submissions are also rejected or
recorded through the stale-submission path when older than the configured
effective maximum age. Persistent entries have their TTL extended while
readings are updated; this retention cap bounds the number of values stored per
key and does not represent a lifetime history.
