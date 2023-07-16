struct PixelInput {
    float4 pos : SV_Position;
    float3 col : COLOR;
};

float4 main(PixelInput IN) : SV_Target
{
    return float4(IN.col, 1.0);
}