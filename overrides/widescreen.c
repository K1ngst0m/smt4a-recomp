/* widescreen: the 3D scene drawn wider than the console's 5:3 top screen,
 * the 2D drawn at its own shape in the middle of it.
 *
 * The game draws into 400x240 whatever the shape, and Zakuro's "Top screen
 * shape" setting stretches that out to the shape. So the 3D is drawn wider
 * than 5:3 by as much as it is stretched, and comes out unstretched with
 * more to the sides (Hor+), while the 2D is drawn narrower by as much and
 * comes out at its own shape, centred.
 *
 * The host sets smt4a_wide to the shape's aspect, 0 for the console's.
 *
 * 3D. The camera's SetPerspective, f_001523B8(camera, fovy, near, far,
 * aspect), aspect in s3, keeps the aspect at camera+0x34 and builds its
 * matrices with f_0016FBA0(mtx, fovy, aspect, near, far), aspect in s1,
 * which a few others call directly. Both are given 400/240 for the top
 * screen; only that is widened, so the camera's own call into the matrix
 * isn't widened twice and other projections are left alone. SMT4 has the
 * same two functions, see smt4-recomp's docs/WIDESCREEN.md.
 *
 * 2D. f_0015A050(mtx, left, right, bottom, top, near, far), in s0 to s5,
 * builds an orthographic matrix. The screen's one is (0, 400, 240, 0, ...),
 * pixels from the top left; it's given a wider left and right around the
 * same middle, which draws 0..400 into the middle 400 * 5:3 / aspect.
 */

#include <string.h>
#include "overrides.h"

#define CONSOLE_ASPECT (400.0f / 240.0f)

RECOMP_EXPORT float smt4a_wide = 0.0f;
/* how many times each hook changed something, for checking a run */
RECOMP_EXPORT uint32_t smt4a_wide_counts[4];

static float get_s(Context *ctx, int n) {
    float f;
    memcpy(&f, &ctx->vfp[n], 4);
    return f;
}

static void set_s(Context *ctx, int n, float f) {
    memcpy(&ctx->vfp[n], &f, 4);
}

static int is_console_aspect(float aspect) {
    return aspect > CONSOLE_ASPECT - 0.001f && aspect < CONSOLE_ASPECT + 0.001f;
}

static int widen(Context *ctx, int n) {
    if (smt4a_wide <= CONSOLE_ASPECT || !is_console_aspect(get_s(ctx, n)))
        return 0;
    set_s(ctx, n, smt4a_wide);
    return 1;
}

/* the camera's SetPerspective */
RECOMP_OVERRIDE(0x001523B8) {
    smt4a_wide_counts[0] += widen(ctx, 3);
    CALL(RECOMP_ORIGINAL(0x001523B8));
}

/* the perspective matrix */
static void seen(uint32_t lr, float aspect);

RECOMP_OVERRIDE(0x0016FBA0) {
    seen(ctx->r[14], get_s(ctx, 1));
    smt4a_wide_counts[1] += widen(ctx, 1);
    CALL(RECOMP_ORIGINAL(0x0016FBA0));
}

/* the orthographic matrix */
RECOMP_OVERRIDE(0x0015A050) {
    float left = get_s(ctx, 0), right = get_s(ctx, 1);
    if (smt4a_wide > CONSOLE_ASPECT && left == 0.0f && right == 400.0f) {
        float half = 200.0f * smt4a_wide / CONSOLE_ASPECT;
        set_s(ctx, 0, 200.0f - half);
        set_s(ctx, 1, 200.0f + half);
        smt4a_wide_counts[2]++;
    }
    CALL(RECOMP_ORIGINAL(0x0015A050));
}

/* every (caller, aspect) the perspective matrix is given, for finding the
 * scenes' other projections; replay prints and clears it at each dump */
RECOMP_EXPORT uint32_t smt4a_wide_seen[64][3];
RECOMP_EXPORT uint32_t smt4a_wide_seen_count;

static void seen(uint32_t lr, float aspect) {
    uint32_t bits;
    memcpy(&bits, &aspect, 4);
    for (uint32_t i = 0; i < smt4a_wide_seen_count; i++)
        if (smt4a_wide_seen[i][0] == lr && smt4a_wide_seen[i][1] == bits) {
            smt4a_wide_seen[i][2]++;
            return;
        }
    if (smt4a_wide_seen_count < 64) {
        smt4a_wide_seen[smt4a_wide_seen_count][0] = lr;
        smt4a_wide_seen[smt4a_wide_seen_count][1] = bits;
        smt4a_wide_seen[smt4a_wide_seen_count][2] = 1;
        smt4a_wide_seen_count++;
    }
}

/* the 2D quads, f_0019E5E0 and f_00186244(.., x, y, w, h, u, v, du, dv, ..)
 * in s0 to s7, logged by (caller, x, y, w, h) for finding the full screen
 * ones; replay prints and clears it at each dump */
RECOMP_EXPORT uint32_t smt4a_quads[128][6];
RECOMP_EXPORT uint32_t smt4a_quad_count;

static void quad(Context *ctx) {
    uint32_t key[5] = {ctx->r[14], ctx->vfp[0], ctx->vfp[1], ctx->vfp[2], ctx->vfp[3]};
    for (uint32_t i = 0; i < smt4a_quad_count; i++)
        if (!memcmp(smt4a_quads[i], key, sizeof key)) {
            smt4a_quads[i][5]++;
            return;
        }
    if (smt4a_quad_count < 128) {
        memcpy(smt4a_quads[smt4a_quad_count], key, sizeof key);
        smt4a_quads[smt4a_quad_count][5] = 1;
        smt4a_quad_count++;
    }
}

/* a quad from the left edge of the 0..400 screen to its right edge, or
 * past both (fades and filters at 0..400, the dialogue backdrops at
 * -22..422, the screen drawn back under a menu), is meant to cover the
 * picture: it's spread out around the middle by as much as the screen's
 * ortho was widened, so it covers the widened width and overhangs it as
 * far as it overhung the console's. its texture, if any, is compressed
 * around u=0.5 so the middle part fills the quad and the sides are cropped.
 * a quad short of either edge is UI, and stays its own shape */
static void fill(Context *ctx) {
    float x = get_s(ctx, 0), w = get_s(ctx, 2);
    if (smt4a_wide <= CONSOLE_ASPECT || x > 0.0f || x + w < 400.0f)
        return;
    float k = smt4a_wide / CONSOLE_ASPECT;
    set_s(ctx, 0, 200.0f + (x - 200.0f) * k);
    set_s(ctx, 2, w * k);
    float u = get_s(ctx, 4), du = get_s(ctx, 6);
    set_s(ctx, 4, 0.5f + (u - 0.5f) / k);
    set_s(ctx, 6, du / k);
    smt4a_wide_counts[3]++;
}

RECOMP_OVERRIDE(0x0019E5E0) {
    quad(ctx);
    fill(ctx);
    CALL(RECOMP_ORIGINAL(0x0019E5E0));
}

RECOMP_OVERRIDE(0x00186244) {
    quad(ctx);
    fill(ctx);
    CALL(RECOMP_ORIGINAL(0x00186244));
}
