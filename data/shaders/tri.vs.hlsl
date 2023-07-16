
static float2 positions[3] = {
    float2( 0.0,  0.5),
    float2( 0.5, -0.5),
    float2(-0.5, -0.5),
};

static float3 colors[3] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0),
};

struct VertexOutput
{
    float4 pos : SV_Position;
    float3 col : COLOR;
};

VertexOutput main(uint vertexID : SV_VertexID)
{
    VertexOutput OUT = (VertexOutput)0;

    OUT.pos = float4(positions[vertexID], 0.0f, 1.0f);
    OUT.col = colors[vertexID];

    return OUT;
}