#ifndef _10BLIB_H
#define _10BLIB_H
struct _ImgYUV
{
   int width;
   int height;
   unsigned char *datY;
   unsigned char *datU;
   unsigned char *datV;
};
typedef struct _ImgYUV ImgYUV;

ImgYUV *  initImgYUV(unsigned int width, unsigned int height);

int grabFrame(ImgYUV * img, FILE * fl);
int firstFrame(ImgYUV * img, FILE * fl);
int prevFrame(ImgYUV * img, FILE * fl);
void freeImgYUV(ImgYUV ** img);
int writeFrame(ImgYUV * img, FILE * fl);
void diffYUV(ImgYUV * A, ImgYUV * B, ImgYUV * diff, int c);
void getch(ImgYUV * A, ImgYUV * D, int c);
void readSeed();
void convertToOverlay(ImgYUV * in, SDL_Overlay * out);
#endif
