#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <assert.h>
#include "SDL.h"

#include "imgyuv.h"


ImgYUV *  initImgYUV(unsigned int width, unsigned int height)
{
  ImgYUV * img;
  img = (ImgYUV *) malloc(sizeof(ImgYUV));
  if (img == NULL)
  {
    fprintf(stderr, "\nImgYUV malloc failed\n");
    exit(EXIT_FAILURE);
  }
  (img)->height = height;
  (img)->width = width;

  (img)->datY = (unsigned char *)malloc(width * height * sizeof(unsigned char));
  (img)->datU = (unsigned char *)malloc(((width * height) << 2) * sizeof(unsigned char));
  (img)->datV = (unsigned char *)malloc(((width * height) << 2) * sizeof(unsigned char));

  /*(img)->datY = (unsigned char *)malloc(((width * height * 3)/2) * sizeof(unsigned char)) ;*/
  /*(img)->datU = img->datU + ((width * height)/4) ;*/
  /*(img)->datV = img->datU + ((width * height)/4) ;*/

  if ((img)->datY == NULL || (img)->datU == NULL || (img)->datV == NULL)
  {
    fprintf(stderr, "\nImgYUV malloc failed\n");
    exit(EXIT_FAILURE);
  }
  return img;
}

int firstFrame(ImgYUV * img, FILE * fl)
{
  fseek(fl, 0,    SEEK_SET);
  grabFrame(img, fl);
  return 0;
}

int prevFrame(ImgYUV * img, FILE * fl)
{
  /*fseek(fl, (-2) * (img->width*img->height + img->width*img->height>>1), SEEK_CUR);*/
  fseek(fl, -2 * img->width*img->height,    SEEK_CUR);
  fseek(fl, -2 * img->width*img->height>>2, SEEK_CUR);
  fseek(fl, -2 * img->width*img->height>>2, SEEK_CUR);
  grabFrame(img, fl);
  return 0;
}

int grabFrame(ImgYUV * img, FILE * fl)
{
  if(!fread(img->datY, img->width*img->height,    1, fl)) return 0;
  if(!fread(img->datU, img->width*img->height>>2, 1, fl)) return 0;
  if(!fread(img->datV, img->width*img->height>>2, 1, fl)) return 0;
  /*if(!fread(img->datY, (img->width*img->height*3)/2,    1, fl)) return 0;*/
  return 1;
}

int writeFrame(ImgYUV * img, FILE * fl)
{
  if(!fwrite(img->datY, img->width*img->height,    1, fl)) return 0;
  if(!fwrite(img->datU, img->width*img->height>>2, 1, fl)) return 0;
  if(!fwrite(img->datV, img->width*img->height>>2, 1, fl)) return 0;
  /*if(!fwrite(img->datY, (img->width*img->height * 3)/2,    1, fl)) return 0;*/
  return 1;
}

void getch(ImgYUV * A, ImgYUV * D, int c)
{
  unsigned char *lineY_A;
  unsigned char *lineU_A;
  unsigned char *lineV_A;
  unsigned char *lineY_D;
  unsigned char *lineU_D;
  unsigned char *lineV_D;
  int x, y;

  for (y = 0; y < A->height; y++)
  {
    lineY_A = (unsigned char *)(A->datY + y * A->width);
    lineU_A = (unsigned char *)(A->datU + (y/2) * (A->width/2));
    lineV_A = (unsigned char *)(A->datV + (y/2) * (A->width/2));
    lineY_D = (unsigned char *)(D->datY + y * D->width);
    lineU_D = (unsigned char *)(D->datU + (y/2) * (D->width/2));
    lineV_D = (unsigned char *)(D->datV + (y/2) * (D->width/2));

    for (x = 0; x < A->width; x++)
    {

      if(c == 0)
      {
        lineY_D[x] =   lineY_A[x];
      }
      else if(c == 1)
      {
        lineY_D[x] =   lineU_A[x/2];
      }
      else
      {
        lineY_D[x] =   lineV_A[x/2];
      }
      /*if((lineY_A[x] - lineY_B[x]) != 0)*/
        /*printf("adasd\n");*/
      lineU_D[x/2] = 128;
      lineV_D[x/2] = 128;
      /*lineU_D[x/2] = (5 * (lineU_A[x/2] - lineU_B[x/2])) + 128;*/
      /*lineV_D[x/2] = (5 * (lineV_A[x/2] - lineV_B[x/2])) + 128;*/
    }
  }

}

void diffYUV(ImgYUV * A, ImgYUV * B, ImgYUV * D, int c)
{
  unsigned char *lineY_A;
  unsigned char *lineU_A;
  unsigned char *lineV_A;
  unsigned char *lineY_B;
  unsigned char *lineU_B;
  unsigned char *lineV_B;
  unsigned char *lineY_D;
  unsigned char *lineU_D;
  unsigned char *lineV_D;
  int x, y;

  for (y = 0; y < A->height; y++)
  {
    lineY_A = (unsigned char *)(A->datY + y * A->width);
    lineU_A = (unsigned char *)(A->datU + (y/2) * (A->width/2));
    lineV_A = (unsigned char *)(A->datV + (y/2) * (A->width/2));
    lineY_B = (unsigned char *)(B->datY + y * B->width);
    lineU_B = (unsigned char *)(B->datU + (y/2) * (B->width/2));
    lineV_B = (unsigned char *)(B->datV + (y/2) * (B->width/2));
    lineY_D = (unsigned char *)(D->datY + y * D->width);
    lineU_D = (unsigned char *)(D->datU + (y/2) * (D->width/2));
    lineV_D = (unsigned char *)(D->datV + (y/2) * (D->width/2));

    for (x = 0; x < A->width; x++)
    {

      if(c == 0)
      {
        /*lineY_D[x] =   (5 * (lineY_A[x] - lineY_B[x])) + 128;*/
        lineY_D[x] =   abs(5 * (lineY_A[x] - lineY_B[x])) + 0;
        lineU_D[x/2] = 128;
        lineV_D[x/2] = 128;
        /*lineY_D[x] =   lineY_A[x];*/
      }
      else if(c == 1)
      {
        lineY_D[x] =   abs(5 * (lineU_A[x/2] - lineU_B[x/2])) + 0;
        lineU_D[x/2] = 128;
        lineV_D[x/2] = 128;
        /*lineY_D[x] =   lineU_A[x/2];*/
      }
      else if(c == 2)
      {
        lineY_D[x] =   abs(5 * (lineV_A[x/2] - lineV_B[x/2])) + 0;
        lineU_D[x/2] = 128;
        lineV_D[x/2] = 128;
        /*lineY_D[x] =   lineV_A[x/2];*/
      }
      else
      {
        lineY_D[x]   = abs(5 * (lineY_A[x]   - lineY_B[x]))   + 0;
        lineU_D[x/2] = (5 * (lineU_A[x/2] - lineU_B[x/2])) + 128;
        lineV_D[x/2] = (5 * (lineV_A[x/2] - lineV_B[x/2])) + 128;
      }
      /*if((lineY_A[x] - lineY_B[x]) != 0)*/
        /*printf("adasd\n");*/
      /*lineU_D[x/2] = (5 * (lineU_A[x/2] - lineU_B[x/2])) + 128;*/
      /*lineV_D[x/2] = (5 * (lineV_A[x/2] - lineV_B[x/2])) + 128;*/
    }
  }

}


void freeImgYUV(ImgYUV ** img)
{
  free((*img)->datY);
  free((*img)->datU);
  free((*img)->datV);
  free(*img);
  *img = NULL;
}

void convertToOverlay(ImgYUV * in, SDL_Overlay * out)
{
  int y;

  unsigned char *lineIn;
  unsigned char *lineOut;
  SDL_LockYUVOverlay(out);
  for (y = 0; y < in->height; y++)
  {
    lineIn = (unsigned char *)(in->datY + y * in->width);
    lineOut = out->pixels[0] + y * out->pitches[0];
    memcpy(lineOut, lineIn, out->pitches[0]);
  }

  for (y = 0; y < in->height / 2; y++)
  {
    lineIn = (unsigned char *)(in->datU + y * (in->width / 2));
    lineOut = out->pixels[1] + y * out->pitches[1];
    memcpy(lineOut, lineIn, out->pitches[1]);
  }

  for (y = 0; y < in->height / 2; y++)
  {
    lineIn = (unsigned char *)(in->datV + y * (in->width / 2));
    lineOut = out->pixels[2] + y * out->pitches[2];
    memcpy(lineOut, lineIn, out->pitches[2]);
  }
  SDL_UnlockYUVOverlay(out);
}

