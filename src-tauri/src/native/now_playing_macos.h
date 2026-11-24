#ifndef NOW_PLAYING_MACOS_H
#define NOW_PLAYING_MACOS_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool success;
    bool is_spotube;
    bool is_playing;
    char* app_name;
    char* bundle_id;
    char* title;
    char* artist;
    char* album;
    char* artwork_data_url;
    double duration_secs;
    double elapsed_secs;
    double timestamp_epoch;
    double playback_rate;
} MacNowPlayingInfo;

MacNowPlayingInfo* mac_now_playing_copy_info(void);
void mac_now_playing_free_info(MacNowPlayingInfo* info);
bool mac_now_playing_send_command(int command);

#ifdef __cplusplus
}
#endif

#endif
