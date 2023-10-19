The Vulkan RAL requires certain features to be available on physical devices, or it will otherwise not support the given physical device

Most of there features should generally be available on PC (windows/linux), but might not be available on android.

Apple devices are not expected to supported via MoltenVK and if support is added to onca, this would likely be via a Metal RAL.

Currently, ReBAR is expected to be enabled when using an AMD card, as it this is not enabled, AMD drivers report a 3rd memory heap that we aren't supported yet at the moment