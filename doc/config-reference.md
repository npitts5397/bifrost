## Configuration reference

Bifrost

```yaml
# Bifrost section [optional!]
#
# Contains bifrost server settings
# [usually omitted, to use defaults]
bifrost:
  # name of yaml file to write state database to
  state_file: "state.yaml"

  # name of x509 certificate for https
  #
  # if this file is missing, bifrost will generate one for you
  #
  # if this file exists, bifrost will check that the mac address
  # matches the specified server mac address
  #
  # to generate a fresh certificate, rename/move this file
  # (this might require pairing the Hue App again)
  cert_file: "cert.pem"

# Bridge section
#
# Settings for hue bridge emulation
bridge:
  name: Bifrost
  mac: 00:11:22:33:44:55
  ipaddress: 10.0.0.12
  netmask: 255.255.255.0
  gateway: 10.0.0.1
  timezone: Europe/Copenhagen

  # HTTP port for emulated bridge
  #
  # beware: most client programs do NOT support non-standard ports.
  # This is for advanced users (e.g. bifrost behind a reverse proxy)
  http_port: 80

  # HTTPS port for emulated bridge
  #
  # beware: most client programs do NOT support non-standard ports.
  # This is for advanced users (e.g. bifrost behind a reverse proxy)
  https_port: 443

  # DTLS port for emulated bridge (Hue Entertainment streaming)
  #
  # beware: client programs do NOT support non-standard ports.
  # For advanced users (e.g. bifrost behind a port forwarded firewall)
  entm_port: 2100

  # Send a DTLS HelloVerifyRequest (cookie exchange) on the entertainment
  # port, like a real Hue bridge. Some clients expect it.
  #
  # Set to false to skip the cookie exchange, as older versions did.
  entm_cookie_exchange: true

# Zigbee2mqtt section
#
# Make a sub-section for each zigbee2mqtt server you want to connect
#
# Server names ("some-server", "other-with-tls") identify Z2M backends.
# These names are stored with imported groups and devices, so keep them stable.
# Renaming a server can cause ownership mismatches and duplicate Hue resources.
#
# NOTE: Be sure to use DIFFERENT names for different servers.
# Otherwise the yaml parser will consider it the same server!
z2m:
  some-server:
    # The websocket url for z2m, starting with "ws://".
    #
    # For z2m version 2.x, the url must end in `/api?token=<token>`.
    # For z2m version 1.x, this is optional, but supported.
    #
    # Therefore, Bifrost will adjust the urls if needed.
    # A message will be logged with the rewritten url if this happens.
    #
    # NOTE: The z2m default token is literally the string "your-secret-token",
    # so if unsure, append "/api?token=your-secret-token".
    #
    # Example:
    #
    #   If your z2m frontend is listening on 10.00.0.100:8080, this
    #   is the resuling config:
    #
    url: ws://10.00.0.100:8080/api?token=your-secret-token

  other-with-tls:
    # This will work, but Bifrost will generate a warning that the url has been
    # adapted to include "/api?token=your-secret-token".
    #
    # NOTE: Using "wss://" instead of "ws://" enables TLS for this connection.
    url: wss://10.10.0.102:8080

    # Disable TLS verify [optional!]
    #
    # If this parameter is included, and has a value of "true", TLS certificate
    # verification will be disabled!
    #
    # NOTE: From a security standpoint, this is almost as bad as disabling
    # encryption entirely. If having a secure connection is important to you,
    # DO NOT enable this option.
    #
    # If you're using self-signed certificates, enabling this option will allow
    # Bifrost to connect to your z2m server.
    disable_tls_verify: false

    # Group prefix [optional!]
    #
    # If you specify this parameter, *only* groups with this prefix
    # will be visible from this z2m server. The prefix will be removed.
    #
    # Example:
    #
    #   With a group_prefix of "bifrost_", the group "bifrost_kitchen"
    #   will be available as "kitchen", but the group "living_room" will
    #   be hidden instead.
    #
    group_prefix: bifrost_

    # Streaming mode ("Entertainment mode" / "Hue Sync") maximum frames per second
    # [optional!]
    #
    # This is the maximum number of light updates attempted per second.
    #
    # The incoming data stream (from a Sync Box, Hue Sync for Windows/Mac,
    # or some other client) determines the maximum possible fps.
    #
    # For example, if Bifrost only receives light updates at 10 fps, setting
    # this limit to 20 will still only cause the lights to update at 10 fps.
    #
    # On the other hand, if the streaming client sends faster than this limit,
    # frames will be dropped to avoid going over it.
    #
    # If not specified, uses a default of 20, which is an attempt to balance
    # responsiveness against load on the Zigbee mesh.
    #
    # Because of the smoothing algorithm Bifrost uses, the results will look
    # *better* if this is not set higher than needed.
    #
    # For example, 30 fps content will look good at 10, 20 or 30 streaming_fps,
    # but worse at streaming_fps: 60, because the frame-to-frame transition
    # time will be wrong for the content.
    #
    # Rules of thumb(s), for best results:
    # - Higher numbers mean greater load on your Zigbee mesh.
    # - If your mesh starts lagging or becoming unresponsive, try a lower number.
    # - Even values as low as 5 fps looks pretty good.
    # - There usually no reason to go above 60.
    # - Have fun experimenting :-)
    streaming_fps: 20

    # Streaming mode ("Entertainment mode" / "Hue Sync") maximum number of
    # light records per Zigbee message
    # [optional!]
    #
    # Every frame of an entertainment stream contains one record per
    # entertainment channel (one per light, or one per segment for
    # multi-segment lights such as the Hue Signe or gradient lightstrips).
    # A Zigbee message can only carry so many records. Frames with more records
    # than this limit are split into several, evenly sized, messages that are
    # sent back to back (so a 14-record frame becomes 4 messages of 4, 4, 3
    # and 3 records).
    #
    # If not specified, uses a default of 4. Messages with 6 records have been
    # observed to be rejected by Zigbee2MQTT with MESSAGE_TOO_LONG.
    #
    # Rules of thumb(s):
    # - If you see MESSAGE_TOO_LONG errors from Zigbee2MQTT, lower this.
    # - Higher values mean fewer Zigbee messages per frame, and therefore
    #   less load on your Zigbee mesh and adapter, but only as long as your
    #   adapter accepts them.
    streaming_max_records_per_message: 4

    # Streaming mode ("Entertainment mode" / "Hue Sync") maximum number of
    # light records sent for each frame
    # [optional!]
    #
    # Lights with many segments (such as the Hue Signe) need many records per
    # frame, which means many Zigbee messages per second. If this causes
    # trouble for your Zigbee mesh or adapter (for example "out of buffers" or
    # ACK timeout errors from Zigbee2MQTT), this option reduces the traffic
    # by sending only some of the records of each frame.
    #
    # The records that were sent the longest time ago go first, so all
    # records are updated in turn. For example, with 14 records and a limit
    # of 4, every record is updated on every 4th frame. Bifrost lengthens the
    # fade of the lights to match, so the result stays smooth, but each light
    # segment reacts more slowly.
    #
    # If not specified, every record is sent for every frame.
    streaming_max_records_per_frame: 4

    # Streaming mode ("Entertainment mode" / "Hue Sync") minimum change
    # [optional!]
    #
    # A number from 1 to 100 (percent). A record is only sent again once its
    # brightness or color has changed by at least this much since it was last
    # sent (records that did not change are sent again about once per second,
    # to keep the lights from going stale).
    #
    # This helps most with slow or partly static effects. For effects where
    # all segments change all the time, it saves little.
    #
    # Can be combined with `streaming_max_records_per_frame`, in which case
    # only changed records are sent, up to that limit.
    #
    # If not specified, every record is sent for every frame.
    streaming_min_change_percent: 2

    # Streaming mode ("Entertainment mode" / "Hue Sync") maximum number of
    # segments per light
    # [optional!]
    #
    # Lights with many segments (the Hue Signe has 10, gradient lightstrips
    # have 7) are shown to clients as one channel per segment, and every
    # channel takes a record in every frame. This option merges neighbouring
    # segments into at most this many groups, so that such a light has fewer
    # channels and uses fewer records per frame.
    #
    # For example, with 3, a 10 segment light is shown as 3 channels that cover
    # 4, 3 and 3 of its segments. Lights with fewer segments are left alone.
    #
    # This applies to every multi-segment light, and takes effect when the
    # entertainment areas are saved again (e.g. by editing the area in the Hue
    # app), since the channels of an area are fixed when it is saved.
    #
    # Note: when streaming, the lights are told to treat all the segments of a
    # group as one, by giving them the same address. This has only been tried
    # on a Hue Signe. If some segments stay dark, remove this option.
    #
    # If not specified, every segment is a channel.
    streaming_max_segments_per_light: 3

    # Streaming mode ("Entertainment mode" / "Hue Sync") statistics
    # [optional!]
    #
    # Log statistics about the entertainment stream every this many seconds
    # (frames in, records and Zigbee messages sent per second, and the longest
    # time between two updates of a record, in frames). Useful for comparing
    # settings, since it shows what a setting did to the traffic.
    #
    # Whatever this is set to, a summary of the same numbers is logged when a
    # stream ends, and the streaming settings in effect are logged when
    # Bifrost starts.
    #
    # If not specified, only the summary and the settings are logged.
    streaming_stats_interval_secs: 10

    # Only show Hue devices
    # [optional!]
    #
    # If true, devices that are not made by Hue (Signify) are left out of
    # Bifrost entirely: they are not shown in the Hue app, and are not part of
    # the rooms and zones that contain them. They still work in Zigbee2MQTT,
    # including as part of its groups.
    #
    # Hue devices are recognized by their manufacturer, not by their model, so
    # new Hue devices are shown without Bifrost having to know them.
    #
    # Devices that were added by an earlier run are removed again.
    #
    # If not specified (or false), every device is shown.
    only_hue_devices: true

    # The icon of devices in the Hue app
    # [optional!]
    #
    # The icon of a device is set by its "archetype". Bifrost knows the
    # archetype of many Hue models, and guesses the archetype of other Hue
    # lights from their description in Zigbee2MQTT. Use this to set the
    # archetype yourself, by the friendly name or by the model of a device.
    # A friendly name takes precedence over a model.
    #
    # Examples of archetypes: classic_bulb, sultan_bulb, candle_bulb,
    # spot_bulb, recessed_ceiling, ceiling_round, pendant_round, floor_shade,
    # table_shade, hue_signe, hue_lightstrip, hue_play, plug
    device_archetypes:
      "Living Room lamp": table_shade
      "VZM31-SN": ceiling_round
  ...

# In-memory Zigbee2MQTT test server [optional!]
#
# This starts a local WebSocket server that mimics the Z2M topic/payload API.
# Configure a regular z2m server above with `url: ws://127.0.0.1:18080/api`
# to make Bifrost use it.
memory2mqtt:
  listen: 127.0.0.1:18080
  # Optional initial state
  state: {}

# Rooms section [optional!]
#
# This section allows you to map zigbee2mqtt "friendly names" to
# a human-readable description you provide.
#
# Each entry under "rooms" must match a zigbee2mqtt "friendly name",
# and can contain the following keys: (both are optional)
#
#   name: The human-readable name presented in the API (for the Hue App, etc)
#
#   icon: The icon to use for this room. Must be selected from the following
#         list of icons supported by the Hue App:
#
#         attic balcony barbecue bathroom bedroom carport closet computer dining
#         downstairs driveway front_door garage garden guest_room gym hallway
#         home kids_bedroom kitchen laundry_room living_room lounge man_cave
#         music nursery office other pool porch reading recreation staircase
#         storage studio terrace toilet top_floor tv upstairs
#
rooms:
  office_group:
    name: Office 1
    icon: office

  carport_group:
    name: Carport Lights
    icon: carport

  ...
```
