#include <libavcodec/avcodec.h>

const int packetSize = 600 + AV_INPUT_BUFFER_PADDING_SIZE;

int decode_buffer_arr(int bufferArr[]) {

    const AVCodec *videoCodec = avcodec_find_decoder(AV_CODEC_ID_HEVC);

    AVCodecContext *codecContext = avcodec_alloc_context3(videoCodec);
    avcodec_open2(codecContext, videoCodec, NULL);

    AVFrame *frame = av_frame_alloc();
    
    uint8_t *buf = av_malloc(packetSize);
    int i;
    for (i=0; i<600; i++) {
        buf[i] = bufferArr[i];
    }

    AVPacket *packet = av_packet_alloc();
    int packetFromData = av_packet_from_data(packet, buf, packetSize);

    int sendPacketResponse = avcodec_send_packet(codecContext, packet);

    int recFrameResponse = avcodec_receive_frame(codecContext, frame);

    printf("%s\n", av_err2str(recFrameResponse));
    return 0;
}