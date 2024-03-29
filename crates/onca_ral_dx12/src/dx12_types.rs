use windows::Win32::Foundation::BOOL;

//--------------------------------------------------------------

#[repr(C)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub struct D3D12_FEATURE_DATA_D3D12_OPTIONS19 {
    pub MismatchingOutputDimensionsSupported: BOOL,
    pub SupportedSampleCountsWithNoOutputs: u32,
    pub PointSamplingAddressesNeverRoundUp: BOOL,
    pub RasterizerDesc2Supported: BOOL,
    pub NarrowQuadrilateralLinesSupported: BOOL,
    pub AnisoFilterWithPointMipSupported: BOOL,
    pub MaxSamplerDescriptorHeapSize: u32,
    pub MaxSamplerDescriptorHeapSizeWithStaticSamplers: u32,
    pub MaxViewDescriptorHeapSize: u32,
    pub ComputeOnlyCustomHeapSupported: BOOL,
}
impl ::core::marker::Copy for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {}
impl ::core::clone::Clone for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::core::fmt::Debug for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("D3D12_FEATURE_DATA_D3D12_OPTIONS16")
            .field("MismatchingOutputDimensionsSupported", &self.MismatchingOutputDimensionsSupported)
            .field("SupportedSampleCountsWithNoOutputs", &self.SupportedSampleCountsWithNoOutputs)
            .field("PointSamplingAddressesNeverRoundUp", &self.PointSamplingAddressesNeverRoundUp)
            .field("RasterizerDesc2Supported", &self.RasterizerDesc2Supported)
            .field("NarrowQuadrilateralLinesSupported", &self.NarrowQuadrilateralLinesSupported)
            .field("AnisoFilterWithPointMipSupported", &self.AnisoFilterWithPointMipSupported)
            .field("MaxSamplerDescriptorHeapSize", &self.MaxSamplerDescriptorHeapSize)
            .field("MaxSamplerDescriptorHeapSizeWithStaticSamplers", &self.MaxSamplerDescriptorHeapSizeWithStaticSamplers)
            .field("MaxViewDescriptorHeapSize", &self.MaxViewDescriptorHeapSize)
            .field("ComputeOnlyCustomHeapSupported", &self.ComputeOnlyCustomHeapSupported)
        .finish()
    }
}
impl ::windows::core::TypeKind for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {
    type TypeKind = ::windows::core::CopyType;
}
impl ::core::cmp::PartialEq for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {
    fn eq(&self, other: &Self) -> bool {
        self.MismatchingOutputDimensionsSupported == other.MismatchingOutputDimensionsSupported &&
        self.SupportedSampleCountsWithNoOutputs == other.SupportedSampleCountsWithNoOutputs &&
        self.PointSamplingAddressesNeverRoundUp == other.PointSamplingAddressesNeverRoundUp &&
        self.RasterizerDesc2Supported == other.RasterizerDesc2Supported &&
        self.NarrowQuadrilateralLinesSupported == other.NarrowQuadrilateralLinesSupported &&
        self.AnisoFilterWithPointMipSupported == other.AnisoFilterWithPointMipSupported &&
        self.MaxSamplerDescriptorHeapSize == other.MaxSamplerDescriptorHeapSize &&
        self.MaxSamplerDescriptorHeapSizeWithStaticSamplers == other.MaxSamplerDescriptorHeapSizeWithStaticSamplers &&
        self.MaxViewDescriptorHeapSize == other.MaxViewDescriptorHeapSize &&
        self.ComputeOnlyCustomHeapSupported == other.ComputeOnlyCustomHeapSupported
    }
}
impl ::core::cmp::Eq for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {}
impl ::core::default::Default for D3D12_FEATURE_DATA_D3D12_OPTIONS19 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
