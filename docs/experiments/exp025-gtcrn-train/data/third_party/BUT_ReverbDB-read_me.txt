Copyright 2018 Brno University of Technology, Faculty of Information Technology, BUT Speech@FIT

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

====================================
Citations
====================================
Whenever publishing any output based on this dataset please check https://speech.fit.vutbr.cz/software for proper citation.
Actually:
Building and Evaluation of a Real Room Impulse Response Dataset
Igor Szoke, Miroslav Skacel, Ladislav Mosner, Jakub Paliesek, Jan "Honza" Cernocky
Submitted to Journal of Selected Topics in Signal Processing, November 2018

https://arxiv.org/abs/1811.06795

====================================
Version 0.2 - pre-release beta
====================================
2018.11.21 - Added 5 more rooms (7 in total) - 8th in progress.
2018.09.24 - Debug of some files missing. The tarball should be complete now.
2018.07.11 - This is pre-release version. Feel free to provide us with your feedback to szoke@fit.vutbr.cz and subject mentioning BUT-ReverbDB.

====================================
The philosophy
====================================
This BUT Speech@FIT Reverb Database is being build with respect to collect large number of various Room Impulse Responses, Room environmental noise (or "silence"),
Retransmitted speech (for ASR and SID testing), and metadata.
The goal is to provide the community with dataset for data enhancement and distant mic / microphone array experiments in ASR and SID.

====================================
Known caveats and bugs
====================================

Hotel_SkalskyDvur_ConferenceRoom2 - Weird differences between estimated spk-mic distance and measured one. Do not trust the measured positions. But the RIRs looks good.

Please report anything stange to szoke@fit.vutbr.cz and subject mentioning BUT-ReverbDB.


====================================
Directory structure
====================================
Example: ./VUT_FIT_L207/MicID01/SpkID01_20171223_S/01/RIR/

Description: ./PLACE/MIC_SETUP/SPK_SETUP/MIC_ID/DATASET/DATASET_HIERARCHY

where

* PLACE: Consists of 3 tokens separated by underscores. For example ORGANIZATION_BUILDING_ROOM, HOTEL_NAME_ROOM, CAR_TYPE_MODEL, etc..

* MIC_SETUP: an ID of placing of microphones in the room. Usually there was done only one microphone placing in particular room.

* SPK_SETUP: Consists of 3 tokens separated by underscores.
  ** 1st is an ID of placing of the speaker(s) in the room. Usually there were done more speaker(s) placing in particular room.
  ** 2nd is the date of recording.
  ** 3rd is a tag of what was recorded. There are following tags:
    *** S means just RIR measurements were done. So here you can find only RIR and silence datasets.
    *** T means test data was retransmitted and also (usually) RIR measurements were done. Here you can find RIR, silence and other datasets.

* MIC_ID: ID of particular microphone (audio channel).

* DATASET: Name of a data set. We provide following datasets:
  ** RIR - directory contains wav files with extracted RIRs (compensated for speaker -> microphone delay)
  ** silence - directory contains wav files with recorded "silence" ... meaning background noise
  ** english/LibriSpeech/test-clean/ - directory contains retransmited LibriSpeech - test-clean audios [http://www.openslr.org/12/]. All the audios are compensated for the delay (speaker -> microphone).

All audio files are PCMs signed int, mono, 16bit, 16kHz (wav format). All files contains extension .v[0-9][0-9].wav. The number (2 digits) after the "v" means the recorded version in a sequence. Starting with 00.
For example. We record RIRs and silences during retransmition of LibriSpeech several times. So then there is this counter telling us which "version" it is. One file can be retransmitted several times, and we may want
all the versions.


There are some more files (photos, meta information, scripts).
* ./PLACE/
  ** *.jpg contains set of photos of the room. Usually a set of "panoramic" photos and optionally a set of photos of some microphones placing.
  ** env_meta.txt is a file containing just the room meta information (see below)
  ** env_full_meta.txt is a file containing full meta information of the whole environment -> room, microphones position, speakers position  (see below)
  ** spk_pos.py is a script which visualizes placing and orientation of speakers. The speaker numbering is in format A.B where A is SPK_SETUP ID and B is ID of the speaker in this setup (see below).

* ./PLACE/MIC_SETUP/SPK_SETUP/
  ** spk_meta.txt is a file containing just the room and the speaker(s) meta information (see below)
  ** mic_pos.py is a script which visualizes placing and orientation of microphones regarding to the position of the speaker (the speech source - see below).

* ./PLACE/MIC_SETUP/SPK_SETUP/MIC_ID/
  ** mic_meta.txt is a file containing just the room, the speaker(s) and the microphone meta information (see below)


====================================
META information
====================================
We collected vast amount of meta information describing the environment. This is provided in *.txt files in some reasonable computer readable format. We also pre-calculated some distances and other values for you
to save you time searching for coordinates transformation formulas.


We expect the room to has a block shape. As the absolute starting point (0,0,0), we defined right hand, bottom, back corner after we enter the room (using doors) :). We measure:
* Depth - Back to front
* Width - Right to left
* Height - Down to top
If the room has L shape, then we split it to 2 blocks. We provide several different coordinate systems to make better user experience when using this data set. We use cartesian (depth, width, height) and
spherical (distance, azimuth, elevation) in absolute and relative way for a microphone or a speaker position. We use azimuth and elevation for a microphone or a speaker orientation.
Absolute coordinates are related to the absolute starting point 0,0,0. Absolute orientation is related to the "look front" when we entered the room (vector orthogonal to the back wall (wall having the door).
We respect the clockwise angle so 90 degrees is on your right hand (azimuth) and above your head (elevation).
Relative point is a point in room to make some distances more user friendly. It is usually the placement of the speech speaker. So you can easily get microphone to speaker distance just by looking to
the relative position (distance) of microphone.

====================================
ROOM META information example
====================================

$EnvID  7       the unique measurement id (int)
$EnvName        VUT_FIT_L207    PLACE (string)
$EnvTemp        18.0    temperature deg. Celsius (float)
$EnvDescription         full text description (string)
$EnvDepth       4.585   depth in meters (float)
$EnvWidth       6.903   width in meters (float)
$EnvHeight      3.144   heights in meters (float)
$Env2Depth              depth of the second block of L shaped rooms in meters (float)
$Env2Width              width of the second block of L shaped rooms in meters (float)
$Env2Height             heights of the second block of L shaped rooms in meters (float)
$EnvVolume      99.50840172     calculated volume in cubic meters (float)
$EnvType        Office  type of room (string)
$EnvSubType     Middle  size of room (string)
$EnvBCKNoiseLevel       34.5    measure background noise level in dBA (float)
$EnvMatWall     Concrete, Glass, Plasterboard   material used for walls (string)
$EnvMatFloor    Linoleum        material used for floor (string)
$EnvMatCeiling  Concrete        material used for ceiling (string)
$EnvFurniture   50.0    amount of floor surface (2D! not the volume) occupied by furniture (anything going to the "volume" of the room) in percents (float)
$EnvRPDepth     3.654   position of relative point (depth relative to 0,0,0 point in meters (float)
$EnvRPWidth     1.266   position of relative point (width relative to 0,0,0 point in meters (float)
$EnvRPHeight    1.461   position of relative point (height relative to 0,0,0 point in meters (float)
$EnvRPAzimuth   -90.0   orientation of relative point (azimuth relative to vector orthogonal to back wall oriented to front) in degrees (float)
$EnvRPElevation 0.0     orientation of relative point (elevation relative to vector orthogonal to back wall oriented to front) in degrees (float)


The relative point orientation here means the speaker is turned to left (90 degrees).


====================================
SPEAKER META information example
====================================

$EnvSpkSetupID 1        SPK_SETUP id (int)
$EnvSpkSetupName sitting speaker        tag of what should the speaker simulate (string)
$EnvSpkSetupDescription         description of what should the speaker simulate (string)

$EnvSpk1RelDepth 0.0    position related to relative point in meters (float)
$EnvSpk1RelWidth 0.0    position related to relative point in meters (float)
$EnvSpk1RelHeight 0.0   position related to relative point in meters (float)
$EnvSpk1RelAzimuth 0.0  position related to relative point in degrees (float)
$EnvSpk1RelElevation 0.0        position related to relative point in degrees (float)
$EnvSpk1RelDistance 0.0 position related to relative point in meters (float)
$EnvSpk1DirAzimuth -90.0        orientation (absolute) of the speaker in degrees (float)
$EnvSpk1DirElevation 0.0        orientation (absolute) of the speaker in degrees (float)
$EnvSpk1RelDirAzimuth 0.0       orientation related to relative point direction in degrees (float)
$EnvSpk1RelDirElevation 0.0     orientation related to relative point direction in degrees (float)
$EnvSpk1Type Adam Audio type of the speaker (string)
$EnvSpk1SndGain 4.0     settings on the soundcard (float)
$EnvSpk1SpkGain 2.5     settings on the speaker (float)
$EnvSpk1Depth 3.654     position (absolute) of the speaker in meters (float)
$EnvSpk1Width 1.266     position (absolute) of the speaker in meters (float)
$EnvSpk1Height 1.461    position (absolute) of the speaker in meters (float)
$EnvSpk1Distance 4.133883525209678      position (absolute) of the speaker in meters (float)
$EnvSpk1Azimuth -19.109647128776032     position (absolute) of the speaker in degrees (float)
$EnvSpk1Elevation 20.69668360319063     position (absolute) of the speaker in degrees (float)

Here you can see that the speaker $EnvSpk1* is placed to the relative point (defined in the room). If more speakers $EnvSpk2, $EnvSpk3, .. appears, we used them as a source of
background noise (radio playing, etc..). The speech is played always from the first speaker.


====================================
MICROPHONE META information example
====================================

$EnvMicSetupID 1        MIC_SETUP (int)
$EnvMicSetupName VUT_FIT_D105   tag of mic setup template - not important (string)
$EnvMicSetupDescription description of what should the microphone setup simulate (string)

$EnvMicID 1     MIC_ID (int)
$EnvMic1RelDepth -1.0390480842804788    position related to relative point in meters (float)
$EnvMic1RelWidth 0.9521121509292856     position related to relative point in meters (float)
$EnvMic1RelHeight 0.044289170300160885  position related to relative point in meters (float)
$EnvMic1RelAzimuth -47.5        position related to relative point in degrees (float)
$EnvMic1RelElevation 1.7999999999999998 position related to relative point in degrees (float)
$EnvMic1RelDistance 1.4100000000000001  position related to relative point in degrees (float) - measured distance between the speaker and microphone
$EnvMic1DirAzimuth 42.499999999999986   orientation (absolute) of the speaker in degrees (float)
$EnvMic1DirElevation 30.73972033964696  orientation (absolute) of the speaker in degrees (float)
$EnvMic1RelDirAzimuth 0.0       orientation related to relative point direction in degrees (float)
$EnvMic1RelDirElevation 32.53972033964696       orientation related to relative point direction in degrees (float)
$EnvMic1TypeID 01-1     tag of the microphone - not important (string)
$EnvMic1Type aaa        type (manufacturer) of the microphone (string)
$EnvMic1Mounting Mic Ball – ch. 1       mounting of the microphone (string)
$EnvMic1Placing on furniture    where was the microphone placed (string)
$EnvMic1SpeakerVisibility visible       was there direct visibility of the speaker from the mic? (string)
$EnvMic1RecGain 5.0     gain on sound card (float)
$EnvMic1RecRate 48000.0 recording settings (float)
$EnvMic1RecBits 24      recording settings (float)
$EnvMic1RecFormat PCM   recording settings (string)
$EnvMic1Notes           some other notes (string)
$EnvMic1Width 2.2181121509292856        position (absolute) of the microphone in meters (float)
$EnvMic1Depth 2.614951915719521 position (absolute) of the microphone in meters (float)
$EnvMic1Height 1.505289170300161        position (absolute) of the microphone in meters (float)
$EnvMic1Distance 3.744848531229038      position (absolute) of the microphone in meters (float)
$EnvMic1Azimuth -40.306010575101936     position (absolute) of the speaker in degrees (float)
$EnvMic1Elevation 23.700929517835046    position (absolute) of the speaker in degrees (float)
$EnvMic1RelDistanceRIRMeasured          position related to relative point in degrees (float) - distance between the speaker and microphone estimated from RIR


Here the relative coordinates are related to the speaker. Relative direction of microphone is defined as "looking at the speaker -- relative point" for 0,0 degrees.
Visibility is:
* visible - microphone sees the speaker 1 ($EnvSpkSetupID=1 and $EnvSpk=1). Does not hold for other Speaker Setups!
* non-visible - microphone does not see the speaker 1 - there is an obstacle (chair)
* partly boxed - microphone does not see the speaker 1 - microphone is placed in a shelf, opened waste bin, etc but the surrounding area partly opened.
* fully boxed - microphone does not see the speaker 1 - microphone is placed in a closed drawer, closed waste bin, etc.


