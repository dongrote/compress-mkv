# compress-mkv

A rust tool built to compress mkv files for use with my home media server.
Requires `ffmpeg` installed and accessible via the `PATH`.

## Usage
```
Usage: compress-mkv [OPTIONS] INFILE [OUTFILE]

Compress mkv files for use by Jellyfin/Emby/etc

Parameters:
  INFILE             Input file/directory
  [OUTFILE]          Output file (not implemented)

Options:
  -s, --sample       Transcode a small sample. (not implemented)
  -d, --dry-run      Describe what would be done, but don't actually do anything. (not implemented)
  -r, --recursive    Recurse into subdirectories. (not implemented)
  -c, --codec        Codec to use for compression. [av1, hevc] (default: av1)
  -f, --fast         Use faster encoding parameters.
  -h, --help         Show this help message.
```

## Example

```
user@dev:~$ ./compress-mkv -f test/sample.mkv -c hevc
[src/compressor.rs:20:5] options = CompressorOptions {
    dry_run: false,
    fast: true,
    overwrite: false,
    codec: "hevc",
}
probing "test/sample.mkv"
ffmpeg "-hide_banner" "-nostats" "-loglevel" "warning" "-progress" "pipe:1" "-i" "test/sample.mkv" "-c:v" "libx265" "-crf" "35" "-preset" "faster" "-tag:v" "hvc1" "-c:a" "copy" "-c:s" "copy" "-map" "0" "test/sample.hevc.mkv"
x265 [info]: HEVC encoder version 3.5+1-f0c1022b6
x265 [info]: build info [Linux][GCC 12.2.0][64 bit] 8bit+10bit+12bit
x265 [info]: using cpu capabilities: MMX2 SSE2Fast LZCNT SSSE3 SSE4.2 AVX FMA3 BMI2 AVX2
x265 [info]: Main profile, Level-4 (Main tier)
x265 [info]: Thread pool created using 16 threads
x265 [info]: Slices                              : 1
x265 [info]: frame threads / pool features       : 4 / wpp(12 rows)
x265 [info]: Coding QT: max CU size, min CU size : 64 / 8
x265 [info]: Residual QT: max TU size, max depth : 32 / 1 inter / 1 intra
x265 [info]: ME / range / subpel / merge         : hex / 57 / 2 / 2
x265 [info]: Keyframe min / max / scenecut / bias  : 25 / 250 / 40 / 5.00
x265 [info]: Lookahead / bframes / badapt        : 15 / 4 / 0
x265 [info]: b-pyramid / weightp / weightb       : 1 / 1 / 0
x265 [info]: References / ref-limit  cu / depth  : 2 / on / on
x265 [info]: AQ: mode / str / qg-size / cu-tree  : 2 / 1.0 / 32 / 1
x265 [info]: Rate Control / qCompress            : CRF-35.0 / 0.60
x265 [info]: tools: rd=2 psy-rd=2.00 early-skip rskip mode=1 signhide tmvp
x265 [info]: tools: fast-intra strong-intra-smoothing lslices=4 deblock sao
transcoding 14.3MB:  47%|██████████████████████████████████████████████████████████████████████████                                                                | 2135/4506 [00:09<00:10, 215.70it/s, 262.1kB (553.3kB)]
```
