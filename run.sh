#!/bin/bash
export TMDB_API_KEY="Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiI5NzY2NDE3YTEzZjAyYjllZjIzYTA4OTkzMGFjMjA2NyIsIm5iZiI6MTc2OTEyNjg3NS45OTQsInN1YiI6IjY5NzJiYmRiMDUyNTVmNGIyMWFlZGNkMCIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.ktbwNgSHkp3-8akpbzPigomLJR11DmcUhX9d3a02nmM"
export DATABASE_URL="sqlite:///Users/louis/subroot/coding/streaming/streaming.db"
cd /Users/louis/subroot/coding/streaming && cargo run --release