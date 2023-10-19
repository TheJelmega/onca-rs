
cbuffer Constants : register(b0) {
    float4x4 model;
    float4x4 view;
    float4x4 proj;
};

struct VertexInput {
    float2 pos : POSITION;
    float3 col : COLOR;
};

struct VertexOutput
{
    float4 pos : SV_Position;
    float3 col : COLOR;
};

VertexOutput main(VertexInput IN)
{
    VertexOutput OUT = (VertexOutput)0;

    float4x4 mvp = mul(mul(model, view), proj);
    OUT.pos = mul(float4(IN.pos, 0.0f, 1.0f), mvp);
    OUT.col = IN.col;

    return OUT;
}
