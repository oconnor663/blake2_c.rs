// This file is a mix of code from the official BLAKE2 sse implementation and
// from https://github.com/sneves/blake2-avx2. The original BLAKE2 license is
// preserved below.

/*
   BLAKE2 reference source code package - optimized C implementations

   Copyright 2012, Samuel Neves <sneves@dei.uc.pt>.  You may use this under the
   terms of the CC0, the OpenSSL Licence, or the Apache Public License 2.0, at
   your option.  The terms of these licenses can be found at:

   - CC0 1.0 Universal : http://creativecommons.org/publicdomain/zero/1.0
   - OpenSSL license   : https://www.openssl.org/source/license.html
   - Apache 2.0        : http://www.apache.org/licenses/LICENSE-2.0

   More information about the BLAKE2 hash function can be found at
   https://blake2.net.
*/

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "blake2.h"
#include "blake2-impl.h"

// begin code imported from AVX2
#include "blake2b-common.h"

ALIGN(64) static const uint64_t blake2b_IV[8] = {
  UINT64_C(0x6A09E667F3BCC908), UINT64_C(0xBB67AE8584CAA73B),
  UINT64_C(0x3C6EF372FE94F82B), UINT64_C(0xA54FF53A5F1D36F1),
  UINT64_C(0x510E527FADE682D1), UINT64_C(0x9B05688C2B3E6C1F),
  UINT64_C(0x1F83D9ABFB41BD6B), UINT64_C(0x5BE0CD19137E2179),
};

#define BLAKE2B_G1_V1(a, b, c, d, m) do {     \
  a = ADD(a, m);                              \
  a = ADD(a, b); d = XOR(d, a); d = ROT32(d); \
  c = ADD(c, d); b = XOR(b, c); b = ROT24(b); \
} while(0)

#define BLAKE2B_G2_V1(a, b, c, d, m) do {     \
  a = ADD(a, m);                              \
  a = ADD(a, b); d = XOR(d, a); d = ROT16(d); \
  c = ADD(c, d); b = XOR(b, c); b = ROT63(b); \
} while(0)

#define BLAKE2B_DIAG_V1(a, b, c, d) do {                 \
  d = _mm256_permute4x64_epi64(d, _MM_SHUFFLE(2,1,0,3)); \
  c = _mm256_permute4x64_epi64(c, _MM_SHUFFLE(1,0,3,2)); \
  b = _mm256_permute4x64_epi64(b, _MM_SHUFFLE(0,3,2,1)); \
} while(0)

#define BLAKE2B_UNDIAG_V1(a, b, c, d) do {               \
  d = _mm256_permute4x64_epi64(d, _MM_SHUFFLE(0,3,2,1)); \
  c = _mm256_permute4x64_epi64(c, _MM_SHUFFLE(1,0,3,2)); \
  b = _mm256_permute4x64_epi64(b, _MM_SHUFFLE(2,1,0,3)); \
} while(0)

#if defined(PERMUTE_WITH_SHUFFLES)
  #include "blake2b-load-avx2.h"
#elif defined(PERMUTE_WITH_GATHER)
#else
  #include "blake2b-load-avx2-simple.h"
#endif

#if defined(PERMUTE_WITH_GATHER)
ALIGN(64) static const uint32_t indices[12][16] = {
  { 0,  2,  4,  6, 1,  3,  5,  7, 8, 10, 12, 14, 9, 11, 13, 15},
  {14,  4,  9, 13,10,  8, 15,  6, 1,  0, 11,  5,12,  2,  7,  3},
  {11, 12,  5, 15, 8,  0,  2, 13,10,  3,  7,  9,14,  6,  1,  4},
  { 7,  3, 13, 11, 9,  1, 12, 14, 2,  5,  4, 15, 6, 10,  0,  8},
  { 9,  5,  2, 10, 0,  7,  4, 15,14, 11,  6,  3, 1, 12,  8, 13},
  { 2,  6,  0,  8,12, 10, 11,  3, 4,  7, 15,  1,13,  5, 14,  9},
  {12,  1, 14,  4, 5, 15, 13, 10, 0,  6,  9,  8, 7,  3,  2, 11},
  {13,  7, 12,  3,11, 14,  1,  9, 5, 15,  8,  2, 0,  4,  6, 10},
  { 6, 14, 11,  0,15,  9,  3,  8,12, 13,  1, 10, 2,  7,  4,  5},
  {10,  8,  7,  1, 2,  4,  6,  5,15,  9,  3, 13,11, 14, 12,  0},
  { 0,  2,  4,  6, 1,  3,  5,  7, 8, 10, 12, 14, 9, 11, 13, 15},
  {14,  4,  9, 13,10,  8, 15,  6, 1,  0, 11,  5,12,  2,  7,  3},
};

#define BLAKE2B_ROUND_V1(a, b, c, d, r, m) do {                              \
  __m256i b0;                                                                \
  b0 = _mm256_i32gather_epi64((void *)(m), LOAD128(&indices[r][ 0]), 8);     \
  BLAKE2B_G1_V1(a, b, c, d, b0);                                             \
  b0 = _mm256_i32gather_epi64((void *)(m), LOAD128(&indices[r][ 4]), 8);     \
  BLAKE2B_G2_V1(a, b, c, d, b0);                                             \
  BLAKE2B_DIAG_V1(a, b, c, d);                                               \
  b0 = _mm256_i32gather_epi64((void *)(m), LOAD128(&indices[r][ 8]), 8);     \
  BLAKE2B_G1_V1(a, b, c, d, b0);                                             \
  b0 = _mm256_i32gather_epi64((void *)(m), LOAD128(&indices[r][12]), 8);     \
  BLAKE2B_G2_V1(a, b, c, d, b0);                                             \
  BLAKE2B_UNDIAG_V1(a, b, c, d);                                             \
} while(0)

#define BLAKE2B_ROUNDS_V1(a, b, c, d, m) do { \
  int i;                                      \
  for(i = 0; i < 12; ++i) {                   \
    BLAKE2B_ROUND_V1(a, b, c, d, i, m);       \
  }                                           \
} while(0)
#else /* !PERMUTE_WITH_GATHER */
#define BLAKE2B_ROUND_V1(a, b, c, d, r, m) do { \
  __m256i b0;                                   \
  BLAKE2B_LOAD_MSG_ ##r ##_1(b0);               \
  BLAKE2B_G1_V1(a, b, c, d, b0);                \
  BLAKE2B_LOAD_MSG_ ##r ##_2(b0);               \
  BLAKE2B_G2_V1(a, b, c, d, b0);                \
  BLAKE2B_DIAG_V1(a, b, c, d);                  \
  BLAKE2B_LOAD_MSG_ ##r ##_3(b0);               \
  BLAKE2B_G1_V1(a, b, c, d, b0);                \
  BLAKE2B_LOAD_MSG_ ##r ##_4(b0);               \
  BLAKE2B_G2_V1(a, b, c, d, b0);                \
  BLAKE2B_UNDIAG_V1(a, b, c, d);                \
} while(0)

#define BLAKE2B_ROUNDS_V1(a, b, c, d, m) do {   \
  BLAKE2B_ROUND_V1(a, b, c, d,  0, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  1, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  2, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  3, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  4, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  5, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  6, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  7, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  8, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d,  9, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d, 10, (m));        \
  BLAKE2B_ROUND_V1(a, b, c, d, 11, (m));        \
} while(0)
#endif

#if defined(PERMUTE_WITH_GATHER)
#define DECLARE_MESSAGE_WORDS(m)
#elif defined(PERMUTE_WITH_SHUFFLES)
#define DECLARE_MESSAGE_WORDS(m)                                       \
  const __m256i m0 = _mm256_broadcastsi128_si256(LOADU128((m) +   0)); \
  const __m256i m1 = _mm256_broadcastsi128_si256(LOADU128((m) +  16)); \
  const __m256i m2 = _mm256_broadcastsi128_si256(LOADU128((m) +  32)); \
  const __m256i m3 = _mm256_broadcastsi128_si256(LOADU128((m) +  48)); \
  const __m256i m4 = _mm256_broadcastsi128_si256(LOADU128((m) +  64)); \
  const __m256i m5 = _mm256_broadcastsi128_si256(LOADU128((m) +  80)); \
  const __m256i m6 = _mm256_broadcastsi128_si256(LOADU128((m) +  96)); \
  const __m256i m7 = _mm256_broadcastsi128_si256(LOADU128((m) + 112)); \
  __m256i t0, t1;
#else
#define DECLARE_MESSAGE_WORDS(m)           \
  const uint64_t  m0 = LOADU64((m) +   0); \
  const uint64_t  m1 = LOADU64((m) +   8); \
  const uint64_t  m2 = LOADU64((m) +  16); \
  const uint64_t  m3 = LOADU64((m) +  24); \
  const uint64_t  m4 = LOADU64((m) +  32); \
  const uint64_t  m5 = LOADU64((m) +  40); \
  const uint64_t  m6 = LOADU64((m) +  48); \
  const uint64_t  m7 = LOADU64((m) +  56); \
  const uint64_t  m8 = LOADU64((m) +  64); \
  const uint64_t  m9 = LOADU64((m) +  72); \
  const uint64_t m10 = LOADU64((m) +  80); \
  const uint64_t m11 = LOADU64((m) +  88); \
  const uint64_t m12 = LOADU64((m) +  96); \
  const uint64_t m13 = LOADU64((m) + 104); \
  const uint64_t m14 = LOADU64((m) + 112); \
  const uint64_t m15 = LOADU64((m) + 120);
#endif

#define BLAKE2B_COMPRESS_V1(a, b, m, t0, t1, f0, f1) do { \
  DECLARE_MESSAGE_WORDS(m)                                \
  const __m256i iv0 = a;                                  \
  const __m256i iv1 = b;                                  \
  __m256i c = LOAD(&blake2b_IV[0]);                       \
  __m256i d = XOR(                                        \
    LOAD(&blake2b_IV[4]),                                 \
    _mm256_set_epi64x(f1, f0, t1, t0)                     \
  );                                                      \
  BLAKE2B_ROUNDS_V1(a, b, c, d, m);                       \
  a = XOR(a, c);                                          \
  b = XOR(b, d);                                          \
  a = XOR(a, iv0);                                        \
  b = XOR(b, iv1);                                        \
} while(0)
// end of AVX2 import

/* Some helper functions */
static void blake2b_set_lastnode( blake2b_state *S )
{
  S->f[1] = (uint64_t)-1;
}

static int blake2b_is_lastblock( const blake2b_state *S )
{
  return S->f[0] != 0;
}

static void blake2b_set_lastblock( blake2b_state *S )
{
  if( S->last_node ) blake2b_set_lastnode( S );

  S->f[0] = (uint64_t)-1;
}

static void blake2b_increment_counter( blake2b_state *S, const uint64_t inc )
{
  S->t[0] += inc;
  S->t[1] += ( S->t[0] < inc );
}

/* init xors IV with input parameter block */
int blake2b_init_param( blake2b_state *S, const blake2b_param *P )
{
  size_t i;
  /*blake2b_init0( S ); */
  const unsigned char * v = ( const unsigned char * )( blake2b_IV );
  const unsigned char * p = ( const unsigned char * )( P );
  unsigned char * h = ( unsigned char * )( S->h );
  /* IV XOR ParamBlock */
  memset( S, 0, sizeof( blake2b_state ) );

  for( i = 0; i < BLAKE2B_OUTBYTES; ++i ) h[i] = v[i] ^ p[i];

  S->outlen = P->digest_length;
  return 0;
}


/* Some sort of default parameter block initialization, for sequential blake2b */
int blake2b_init( blake2b_state *S, size_t outlen )
{
  blake2b_param P[1];

  if ( ( !outlen ) || ( outlen > BLAKE2B_OUTBYTES ) ) return -1;

  P->digest_length = (uint8_t)outlen;
  P->key_length    = 0;
  P->fanout        = 1;
  P->depth         = 1;
  store32( &P->leaf_length, 0 );
  store32( &P->node_offset, 0 );
  store32( &P->xof_length, 0 );
  P->node_depth    = 0;
  P->inner_length  = 0;
  memset( P->reserved, 0, sizeof( P->reserved ) );
  memset( P->salt,     0, sizeof( P->salt ) );
  memset( P->personal, 0, sizeof( P->personal ) );

  return blake2b_init_param( S, P );
}

int blake2b_init_key( blake2b_state *S, size_t outlen, const void *key, size_t keylen )
{
  blake2b_param P[1];

  if ( ( !outlen ) || ( outlen > BLAKE2B_OUTBYTES ) ) return -1;

  if ( ( !keylen ) || keylen > BLAKE2B_KEYBYTES ) return -1;

  P->digest_length = (uint8_t)outlen;
  P->key_length    = (uint8_t)keylen;
  P->fanout        = 1;
  P->depth         = 1;
  store32( &P->leaf_length, 0 );
  store32( &P->node_offset, 0 );
  store32( &P->xof_length, 0 );
  P->node_depth    = 0;
  P->inner_length  = 0;
  memset( P->reserved, 0, sizeof( P->reserved ) );
  memset( P->salt,     0, sizeof( P->salt ) );
  memset( P->personal, 0, sizeof( P->personal ) );

  if( blake2b_init_param( S, P ) < 0 )
    return 0;

  {
    uint8_t block[BLAKE2B_BLOCKBYTES];
    memset( block, 0, BLAKE2B_BLOCKBYTES );
    memcpy( block, key, keylen );
    blake2b_update( S, block, BLAKE2B_BLOCKBYTES );
    secure_zero_memory( block, BLAKE2B_BLOCKBYTES ); /* Burn the key from stack */
  }
  return 0;
}

// The implementation of this function has been replaced with the experimental
// AVX2 version from https://github.com/sneves/blake2-avx2.
static void blake2b_compress( blake2b_state *S, const uint8_t block[BLAKE2B_BLOCKBYTES] )
{
  ALIGN(64) uint8_t buffer[BLAKE2B_BLOCKBYTES];
  __m256i a = LOADU(&S->h[0]);
  __m256i b = LOADU(&S->h[4]);
  memcpy(buffer, block, BLAKE2B_BLOCKBYTES);
  BLAKE2B_COMPRESS_V1(a, b, buffer, S->t[0], S->t[1], S->f[0], S->f[1]);
  STOREU(&S->h[0], a);
  STOREU(&S->h[4], b);
}


int blake2b_update( blake2b_state *S, const void *pin, size_t inlen )
{
  const unsigned char * in = (const unsigned char *)pin;
  if( inlen > 0 )
  {
    size_t left = S->buflen;
    size_t fill = BLAKE2B_BLOCKBYTES - left;
    if( inlen > fill )
    {
      S->buflen = 0;
      memcpy( S->buf + left, in, fill ); /* Fill buffer */
      blake2b_increment_counter( S, BLAKE2B_BLOCKBYTES );
      blake2b_compress( S, S->buf ); /* Compress */
      in += fill; inlen -= fill;
      while(inlen > BLAKE2B_BLOCKBYTES) {
        blake2b_increment_counter(S, BLAKE2B_BLOCKBYTES);
        blake2b_compress( S, in );
        in += BLAKE2B_BLOCKBYTES;
        inlen -= BLAKE2B_BLOCKBYTES;
      }
    }
    memcpy( S->buf + S->buflen, in, inlen );
    S->buflen += inlen;
  }
  return 0;
}


int blake2b_final( blake2b_state *S, void *out, size_t outlen )
{
  if( out == NULL || outlen < S->outlen )
    return -1;

  if( blake2b_is_lastblock( S ) )
    return -1;

  blake2b_increment_counter( S, S->buflen );
  blake2b_set_lastblock( S );
  memset( S->buf + S->buflen, 0, BLAKE2B_BLOCKBYTES - S->buflen ); /* Padding */
  blake2b_compress( S, S->buf );

  memcpy( out, &S->h[0], S->outlen );
  return 0;
}


int blake2b( void *out, size_t outlen, const void *in, size_t inlen, const void *key, size_t keylen )
{
  blake2b_state S[1];

  /* Verify parameters */
  if ( NULL == in && inlen > 0 ) return -1;

  if ( NULL == out ) return -1;

  if( NULL == key && keylen > 0 ) return -1;

  if( !outlen || outlen > BLAKE2B_OUTBYTES ) return -1;

  if( keylen > BLAKE2B_KEYBYTES ) return -1;

  if( keylen )
  {
    if( blake2b_init_key( S, outlen, key, keylen ) < 0 ) return -1;
  }
  else
  {
    if( blake2b_init( S, outlen ) < 0 ) return -1;
  }

  blake2b_update( S, ( const uint8_t * )in, inlen );
  blake2b_final( S, out, outlen );
  return 0;
}

int blake2( void *out, size_t outlen, const void *in, size_t inlen, const void *key, size_t keylen ) {
  return blake2b(out, outlen, in, inlen, key, keylen);
}

#if defined(SUPERCOP)
int crypto_hash( unsigned char *out, unsigned char *in, unsigned long long inlen )
{
  return blake2b( out, BLAKE2B_OUTBYTES, in, inlen, NULL, 0 );
}
#endif

#if defined(BLAKE2B_SELFTEST)
#include <string.h>
#include "blake2-kat.h"
int main( void )
{
  uint8_t key[BLAKE2B_KEYBYTES];
  uint8_t buf[BLAKE2_KAT_LENGTH];
  size_t i, step;

  for( i = 0; i < BLAKE2B_KEYBYTES; ++i )
    key[i] = ( uint8_t )i;

  for( i = 0; i < BLAKE2_KAT_LENGTH; ++i )
    buf[i] = ( uint8_t )i;

  /* Test simple API */
  for( i = 0; i < BLAKE2_KAT_LENGTH; ++i )
  {
    uint8_t hash[BLAKE2B_OUTBYTES];
    blake2b( hash, BLAKE2B_OUTBYTES, buf, i, key, BLAKE2B_KEYBYTES );

    if( 0 != memcmp( hash, blake2b_keyed_kat[i], BLAKE2B_OUTBYTES ) )
    {
      goto fail;
    }
  }

  /* Test streaming API */
  for(step = 1; step < BLAKE2B_BLOCKBYTES; ++step) {
    for (i = 0; i < BLAKE2_KAT_LENGTH; ++i) {
      uint8_t hash[BLAKE2B_OUTBYTES];
      blake2b_state S;
      uint8_t * p = buf;
      size_t mlen = i;
      int err = 0;

      if( (err = blake2b_init_key(&S, BLAKE2B_OUTBYTES, key, BLAKE2B_KEYBYTES)) < 0 ) {
        goto fail;
      }

      while (mlen >= step) {
        if ( (err = blake2b_update(&S, p, step)) < 0 ) {
          goto fail;
        }
        mlen -= step;
        p += step;
      }
      if ( (err = blake2b_update(&S, p, mlen)) < 0) {
        goto fail;
      }
      if ( (err = blake2b_final(&S, hash, BLAKE2B_OUTBYTES)) < 0) {
        goto fail;
      }

      if (0 != memcmp(hash, blake2b_keyed_kat[i], BLAKE2B_OUTBYTES)) {
        goto fail;
      }
    }
  }

  puts( "ok" );
  return 0;
fail:
  puts("error");
  return -1;
}
#endif
