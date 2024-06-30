# Information
We are limited to 4 binding groups per pipeline due to GLES-3.1 minimum required limits.
There's no specific limit to how many items per group, only per entire pipeline or shader.

For best performance, we should draw as much as possible using the same bound groups before switching groups away.
For that reason, group 0 should change less than group 1, which should change less than group 2, etc.

# Ideal render pass
An ideal render pass looks like this:
- Set group 0
  - Set group 1
    - Set group 2
      - Draw draw draw
    - Set group 2
      - Draw draw draw
  - Set group 1
    - Set group 2
      - Draw draw draw

# Shared
## Group 0: Render pass globals
These should be set for the whole render pass and be immutable during it.

| Index | Type    | Description | Availability |
|:-----:|---------|:------------|--------------|
|   0   | uniform | View matrix | Vertex       |

## Group 1: Mesh transforms
These should be set for the current mesh being rendered.

| Index | Type    | Description       | Availability |
|:-----:|---------|:------------------|--------------|
|   0   | uniform | World matrix      | Vertex       |

# Bitmaps
## Group 2: Color transforms
| Index | Type       | Description                          | Availability |
|:-----:|------------|:-------------------------------------|--------------|
|   0   | uniform    | Color adjustments                    | Fragment     |

## Group 3: Texture transforms
| Index | Type       | Description                          | Availability |
|:-----:|------------|:-------------------------------------|--------------|
|   0   | uniform    | Transformation matrix of the texture | Vertex       |
|   1   | texture_2d | Texture to be drawn                  | Fragment     |
|   2   | sampler    | Sampler used for the texture         | Fragment     |

# Gradient
## Group 2: Color transforms
| Index | Type       | Description                          | Availability |
|:-----:|------------|:-------------------------------------|--------------|
|   0   | uniform    | Color adjustments                    | Fragment     |

## Group 3: Texture transforms
Index 1 is a storage buffer when supported by the device, or a uniform buffer otherwise.
Storage buffers are more efficient and waste less memory, but are not as widely supported (ie WebGL)

| Index | Type               | Description                           | Availability |
|:-----:|--------------------|:--------------------------------------|--------------|
|   0   | uniform            | Transformation matrix of the gradient | Vertex       |
|   1   | uniform or storage | Gradient information, colors etc      | Fragment     |

# Color
## Group 2: Color transforms
| Index | Type       | Description                          | Availability |
|:-----:|------------|:-------------------------------------|--------------|
|   0   | uniform    | Color adjustments                    | Fragment     |
