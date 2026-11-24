#import <Foundation/Foundation.h>
#include "now_playing_macos.h"

typedef void (*MRMediaRemoteGetNowPlayingClientFunction)(dispatch_queue_t, void (^)(id));
typedef void (*MRMediaRemoteGetNowPlayingApplicationIsPlayingFunction)(dispatch_queue_t, void (^)(Boolean));
typedef void (*MRMediaRemoteGetNowPlayingInfoFunction)(dispatch_queue_t, void (^)(CFDictionaryRef));
typedef Boolean (*MRMediaRemoteSendCommandFunction)(int, id);

static CFBundleRef get_media_remote_bundle(void) {
    static CFBundleRef bundle = NULL;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        bundle = CFBundleCreate(kCFAllocatorDefault, (CFURLRef)[NSURL fileURLWithPath:@"/System/Library/PrivateFrameworks/MediaRemote.framework"]);
    });
    return bundle;
}

static char* copy_nsstring_to_utf8(NSString* str) {
    if (!str) return NULL;
    const char* utf8 = [str UTF8String];
    if (!utf8) return NULL;
    return strdup(utf8);
}

MacNowPlayingInfo* mac_now_playing_copy_info(void) {
    MacNowPlayingInfo* result = (MacNowPlayingInfo*)calloc(1, sizeof(MacNowPlayingInfo));
    if (!result) return NULL;

    @autoreleasepool {
        CFBundleRef bundle = get_media_remote_bundle();
        if (!bundle) {
            result->success = false;
            return result;
        }

        MRMediaRemoteGetNowPlayingClientFunction getClient = 
            (MRMediaRemoteGetNowPlayingClientFunction)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteGetNowPlayingClient"));
        MRMediaRemoteGetNowPlayingApplicationIsPlayingFunction isPlayingFn = 
            (MRMediaRemoteGetNowPlayingApplicationIsPlayingFunction)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteGetNowPlayingApplicationIsPlaying"));
        MRMediaRemoteGetNowPlayingInfoFunction getInfo = 
            (MRMediaRemoteGetNowPlayingInfoFunction)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteGetNowPlayingInfo"));

        if (!getClient || !isPlayingFn || !getInfo) {
            result->success = false;
            return result;
        }

        dispatch_queue_t queue = dispatch_queue_create("tlyrics.nowplaying.query", DISPATCH_QUEUE_SERIAL);
        dispatch_semaphore_t sem = dispatch_semaphore_create(0);

        __block NSString* bundleId = @"";
        __block NSString* displayName = @"";
        getClient(queue, ^(id client) {
            if (client) {
                bundleId = [client valueForKey:@"bundleIdentifier"] ?: @"";
                displayName = [client valueForKey:@"displayName"] ?: @"";
            }
            dispatch_semaphore_signal(sem);
        });
        dispatch_semaphore_wait(sem, dispatch_time(DISPATCH_TIME_NOW, 1 * NSEC_PER_SEC));

        __block BOOL playing = NO;
        isPlayingFn(queue, ^(Boolean p) {
            playing = (BOOL)p;
            dispatch_semaphore_signal(sem);
        });
        dispatch_semaphore_wait(sem, dispatch_time(DISPATCH_TIME_NOW, 1 * NSEC_PER_SEC));

        __block NSDictionary* nowInfo = nil;
        getInfo(queue, ^(CFDictionaryRef dict) {
            if (dict) {
                nowInfo = [(__bridge NSDictionary*)dict copy];
            }
            dispatch_semaphore_signal(sem);
        });
        dispatch_semaphore_wait(sem, dispatch_time(DISPATCH_TIME_NOW, 1 * NSEC_PER_SEC));

        result->success = true;
        result->app_name = copy_nsstring_to_utf8(displayName);
        result->bundle_id = copy_nsstring_to_utf8(bundleId);
        result->is_spotube = [bundleId isEqualToString:@"org.spotube.spotube"] || 
                             [displayName rangeOfString:@"Spotube" options:NSCaseInsensitiveSearch].location != NSNotFound;
        result->is_playing = playing;

        if (nowInfo) {
            NSString* title = nowInfo[@"kMRMediaRemoteNowPlayingInfoTitle"];
            NSString* artist = nowInfo[@"kMRMediaRemoteNowPlayingInfoArtist"];
            NSString* album = nowInfo[@"kMRMediaRemoteNowPlayingInfoAlbum"];
            NSNumber* duration = nowInfo[@"kMRMediaRemoteNowPlayingInfoDuration"];
            NSNumber* elapsed = nowInfo[@"kMRMediaRemoteNowPlayingInfoElapsedTime"];
            NSNumber* rate = nowInfo[@"kMRMediaRemoteNowPlayingInfoPlaybackRate"];
            NSDate* timestamp = nowInfo[@"kMRMediaRemoteNowPlayingInfoTimestamp"];
            NSData* artworkData = nowInfo[@"kMRMediaRemoteNowPlayingInfoArtworkData"];
            NSString* mimeType = nowInfo[@"kMRMediaRemoteNowPlayingInfoArtworkMIMEType"] ?: @"image/jpeg";

            result->title = copy_nsstring_to_utf8(title ?: @"");
            result->artist = copy_nsstring_to_utf8(artist ?: @"");
            result->album = copy_nsstring_to_utf8(album ?: @"");
            result->duration_secs = duration ? [duration doubleValue] : 0.0;
            result->elapsed_secs = elapsed ? [elapsed doubleValue] : 0.0;
            result->playback_rate = rate ? [rate doubleValue] : (playing ? 1.0 : 0.0);
            result->timestamp_epoch = timestamp ? [timestamp timeIntervalSince1970] : [[NSDate date] timeIntervalSince1970];

            if (artworkData && [artworkData length] > 0) {
                NSString* b64 = [artworkData base64EncodedStringWithOptions:0];
                NSString* dataUrl = [NSString stringWithFormat:@"data:%@;base64,%@", mimeType, b64];
                result->artwork_data_url = copy_nsstring_to_utf8(dataUrl);
            }
        }
    }
    return result;
}

void mac_now_playing_free_info(MacNowPlayingInfo* info) {
    if (!info) return;
    if (info->app_name) free(info->app_name);
    if (info->bundle_id) free(info->bundle_id);
    if (info->title) free(info->title);
    if (info->artist) free(info->artist);
    if (info->album) free(info->album);
    if (info->artwork_data_url) free(info->artwork_data_url);
    free(info);
}

bool mac_now_playing_send_command(int command) {
    @autoreleasepool {
        CFBundleRef bundle = get_media_remote_bundle();
        if (!bundle) return false;
        MRMediaRemoteSendCommandFunction sendCmd = 
            (MRMediaRemoteSendCommandFunction)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteSendCommand"));
        if (!sendCmd) return false;
        return (bool)sendCmd(command, nil);
    }
}
