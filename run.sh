mkdir tmp;

cargo run -- $1 $2           \
          | ffmpeg -f u16le  \
                   -ar 44100 \
                   -ac 1     \
                   -i pipe:  \
                   -strict experimental \
                   -hls_list_size 3     \
                   -hls_time 2          \
                   -hls_base_url tmp/   \
                   -hls_segment_filename tmp/%03d.ts \
                   -hls_flags delete_segments        \
                   ./playlist.m3u8
