use num_traits::{FromPrimitive};

use crate::{
    AudioFormat,
};

use super::{
    Context,
    AudioDeviceInfo,
    AudioDeviceDirection,

    utils::{
        JNIEnv,
        JObject,
        JResult,
        get_activity,
        with_attached,
        get_system_service,
        get_devices,

        call_method_no_args_ret_int,
        call_method_no_args_ret_string,
        call_method_no_args_ret_char_sequence,
        call_method_no_args_ret_bool,
        call_method_no_args_ret_int_array,
    },
};

impl AudioDeviceInfo {
    /**
     * Request audio devices using Android Java API
     */
    pub fn request(direction: AudioDeviceDirection) -> Result<Vec<AudioDeviceInfo>, String> {
        let activity = get_activity();
        let sdk_version = activity.sdk_version();

        if sdk_version >= 23 {
            with_attached(activity, |env, activity| {
                try_request_devices_info(env, activity, direction)
            }).map_err(|error| error.to_string())
        } else {
            Err("Method unsupported".into())
        }
    }
}

fn try_request_devices_info<'a>(env: &JNIEnv<'a>, activity: JObject, direction: AudioDeviceDirection) -> JResult<Vec<AudioDeviceInfo>> {
    let audio_manager = get_system_service(
        env, activity,
        Context::AUDIO_SERVICE,
    )?;

    let devices = env.auto_local(get_devices(
        &env, audio_manager,
        direction as i32,
    )?);

    let raw_devices = devices.as_obj().into_inner();

    let length = env.get_array_length(raw_devices)?;

    (0..length).into_iter().map(|index| {
        let device = env.get_object_array_element(raw_devices, index)?;

        Ok(AudioDeviceInfo {
            id: call_method_no_args_ret_int(&env, device, "getId")?,
            address: call_method_no_args_ret_string(&env, device, "getAddress")?,
            product_name: call_method_no_args_ret_char_sequence(&env, device, "getProductName")?,
            device_type: FromPrimitive::from_i32(call_method_no_args_ret_int(&env, device, "getType")?).unwrap(),
            direction: AudioDeviceDirection::new(
                call_method_no_args_ret_bool(&env, device, "isSource")?,
                call_method_no_args_ret_bool(&env, device, "isSink")?
            ).ok_or_else(|| "Invalid device direction")?,
            channel_counts: call_method_no_args_ret_int_array(&env, device, "getChannelCounts")?,
            sample_rates: call_method_no_args_ret_int_array(&env, device, "getSampleRates")?,
            formats: call_method_no_args_ret_int_array(&env, device, "getEncodings")?
                .into_iter()
                .map(AudioFormat::from_encoding)
                .filter(Option::is_some)
                .map(Option::unwrap)
                .collect::<Vec<_>>(),
        })
    }).collect::<Result<Vec<_>, _>>()
}
