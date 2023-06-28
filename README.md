# Fast json zip extractor

Basic prototype example of extracting json objects directly out of a gzipped file, as fast as possible through a few tricks:
- streaming the gzip extraction chunk by chunk and processing each chunk
- minimal json parsing of individual sections (individual elements of the array, versus the entire array)

This could be easily extended to write each json chunk to a kafka topic, and would be an ideal candidate to port to AWS Lambda due to the constant memory usage, no matter how large each input file is.

# Testing it yourself

Run `gen.py` in the examples directory first, which will create `example.json` with 500k records in it.

Compress with `gzip example.json`.

Then run the tool with `cat examples/example.json.gz | cargo run`. Pipe the output to /dev/null if you'd just like to see performance metric reporting, for example:
```
cat examples/example.json.gz | cargo run > /dev/null
Processed 10000 records (2.09 seconds elapsed, avg 4782.40 records per second)
Processed 20000 records (4.16 seconds elapsed, avg 4805.38 records per second)
Processed 30000 records (6.38 seconds elapsed, avg 4702.93 records per second)

```

# Benchmark

I haven't got a good idea of what is reasonable for this, but my M1 mac can run this at approx 5,000 records per second (each record approx 1024 bytes each). This is from a source gzipped file.

# Optimisations

Currently does excessive parse attempts - could be tuned to reduce attempts based on likelihood of succeeding rather than brute force approach (while loop).

Could probably seperate the gzipping and json parsing concerns into seperate threads, assuming we want to target multiple cores (?).