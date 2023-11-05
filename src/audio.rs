//use audioviz::io::{Device, Input};
//use bevy::audio::*;
//use bevy::prelude::*;
//
//use audioviz::spectrum::{config::StreamConfig, stream::Stream};
//
//use crate::mandelbrot_material::*;
//
///// Plugin that adds the necessary systems for `PanCamConfig` and `PanCamState` components to work
//#[derive(Default)]
//pub struct AudioVizPlugin;
//
//impl Plugin for AudioVizPlugin {
//    fn build(&self, app: &mut App) {
//        app.init_resource::<MusicUpdateToggle>();
//        app.init_resource::<LastRms>();
//        app.add_systems(Startup, setup_audio);
//        app.add_systems(Update, (update_system_sound_level, update_from_music));
//    }
//}
//
//#[derive(Component)]
//struct MyMusic;
//
//#[derive(Component)]
//struct SystemSoundLevel {
//    value: f32,
//}
//
//#[derive(Resource, Default)]
//pub struct LastRms(f32);
//
//#[derive(Resource)]
//pub struct MusicUpdateToggle {
//    pub active: bool,
//}
//
//impl Default for MusicUpdateToggle {
//    fn default() -> Self {
//        MusicUpdateToggle { active: false }
//    }
//}
//
//fn setup_audio(mut commands: Commands, _asset_server: Res<AssetServer>) {
//    //let music_handle = asset_server.load("audio/1-06 Solitude Is Bliss.flac");
//
//    commands.spawn((
//        //AudioBundle {
//        //    source: music_handle,
//        //    ..Default::default()
//        //},
//        MyMusic,
//        SystemSoundLevel { value: 0.0 }, // New component
//    ));
//}
//
//fn update_from_music(
//    mut materials: ResMut<Assets<MandelbrotMaterial>>,
//    music_controller: Query<(&AudioSink, &SystemSoundLevel), With<MyMusic>>,
//    toggle: Res<MusicUpdateToggle>,
//) {
//    if !toggle.active {
//        return;
//    }
//    for (_audio_sink, sound_level) in music_controller.iter() {
//        if let Some(mandelbrot_material) = materials.iter_mut().next() {
//            mandelbrot_material.1.color_scale = sound_level.value;
//        }
//    }
//}
//
//fn update_system_sound_level(
//    mut sound_level_query: Query<&mut SystemSoundLevel, With<MyMusic>>,
//    last_rms: ResMut<LastRms>,
//    toggle: Res<MusicUpdateToggle>,
//) {
//    if !toggle.active {
//        return;
//    }
//    // Initialize audio input and spectrum visualizer stream just once.
//    // You might consider moving this to a startup system or a resource.
//    let mut audio_input = Input::new();
//    let (_channel_count, _sampling_rate, input_controller) =
//        audio_input.init(&Device::DefaultInput, None).unwrap();
//    let mut stream: Stream = Stream::new(StreamConfig::default());
//    let mut sound_level = 0.0;
//
//    loop {
//        if let Some(data) = input_controller.pull_data() {
//            sound_level = compute_sound_level_from_frequencies(&data, last_rms);
//            stream.push_data(data);
//            stream.update();
//        } else {
//            println!("Failed to pull data");
//        }
//
//        //stream.update();
//        // Retrieve frequencies
//        //let frequencies = stream.get_frequencies();
//        //println!("{}", frequencies[0].len());
//        // Compute sound level from frequencies (this could be RMS or some other measure)
//        //let flattened_frequencies: Vec<Frequency> = frequencies.into_iter().flatten().collect();
//
//        // Update SystemSoundLevel component
//        for mut system_sound_level in sound_level_query.iter_mut() {
//            system_sound_level.value = sound_level;
//        }
//        break;
//    }
//}
//
//fn compute_sound_level_from_frequencies(
//    frequencies: &Vec<f32>,
//    mut last_rms: ResMut<LastRms>,
//) -> f32 {
//    // Define bass frequency range indices
//    let bass_range = 0..50;
//
//    // Filter out only the bass frequencies
//    let bass_frequencies: Vec<f32> = frequencies[bass_range.clone()].to_vec();
//
//    if bass_frequencies.is_empty() {
//        println!("Bass frequencies array is empty");
//        return 0.0;
//    }
//
//    let mut sum_of_squares = 0.0;
//
//    for &frequency in bass_frequencies.iter() {
//        if frequency.is_nan() || frequency.is_infinite() {
//            println!("Invalid frequency value: {}", frequency);
//            continue;
//        }
//        sum_of_squares += frequency.powi(2);
//    }
//
//    if sum_of_squares <= 0.0 {
//        println!("Sum of squares is non-positive: {}", sum_of_squares);
//        return 0.0;
//    }
//
//    let rms = (sum_of_squares / bass_frequencies.len() as f32).sqrt();
//
//    if rms.is_nan() || rms.is_infinite() {
//        println!("Invalid RMS: {}", rms);
//        return 0.0;
//    }
//
//    // EMA smoothing
//    // Alpha is the weight of the new sample, should be between 0 and 1.
//    // Higher alpha discounts older observations faster.
//    let alpha = 0.1;
//    last_rms.0 = alpha * rms + (1.0 - alpha) * last_rms.0;
//
//    // Debugging
//    println!("Smoothed RMS: {}", last_rms.0);
//
//    // Return the smoothed RMS
//    last_rms.0 * 1000.0 + 0.5
//}
