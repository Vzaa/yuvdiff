#include <stdio.h>
#include <sys/time.h>

#include "SDL.h"
#include "SDL_gfxPrimitives.h"
#include "SDL_rotozoom.h"
#include "SDL_ttf.h"

#include "imgyuv.h"

#define ZOOM 24

#define PREV 0

int width;
int height;

int button_w ;
int button_h ;

int end = 0;
int grid = 0;
ImgYUV *img_a;
ImgYUV *img_b;

ImgYUV *chY_a;
ImgYUV *chU_a;
ImgYUV *chV_a;

ImgYUV *chY_b;
ImgYUV *chU_b;
ImgYUV *chV_b;

ImgYUV *diffY;
ImgYUV *diffU;
ImgYUV *diffV;
ImgYUV *diff;
ImgYUV *shown;
int pause = 1;
int update_overlay = 0;

SDL_Rect zoom_s;
SDL_Rect zoom_mb;
SDL_Rect zoom_d;

FILE *reader_a;
FILE *reader_b;

const char button_text[12][24] =  {
    "YUV A (q)",
    "YUV B (a)",
    "YUV Diff (z)",
    "Y A (w)",
    "Y B (s)",
    "Y Diff (x)",
    "U A (e)",
    "U B (d)",
    "U Diff (c)",
    "V A (r)",
    "V B (f)",
    "V Diff (v)",
};

//dump the block data to stdout
void dumpBlock(int x, int y)
{
    unsigned char * mb;
    unsigned char * tmp;
    int offset;
    int i,j;
    offset = ((x >> 4) << 4) + (((y >> 4) * width) << 4);
    mb = shown->datY + offset;
    if(x < width)
    {
        printf("\n");
        printf("\n");
        for (i = 0; i < 16; i++)
        {
            tmp = mb + (width * i);
            for (j = 0; j < 16; j++)
            {
                if(shown == diffY)
                    printf("%03d ", tmp[j] / 5);
                else
                    printf("%03d ", tmp[j]);
            }
            printf("\n");
        }
    }
}

void update_mouse(int x, int y)
{
    if(x < width)
    {
        zoom_s.x = x / 16 * 16 ;
        zoom_s.y = y / 16 * 16 ;
        zoom_s.w = 16;
        zoom_s.h = 16;
    }
}

void rewindFirstFrame()
{
    firstFrame(img_a, reader_a);
    firstFrame(img_b, reader_b);
    diffYUV(img_a, img_b, diffY, 0);
    diffYUV(img_a, img_b, diffU, 1);
    diffYUV(img_a, img_b, diffV, 2);
    diffYUV(img_a, img_b, diff, 3);

    getch(img_a, chY_a, 0);
    getch(img_a, chU_a, 1);
    getch(img_a, chV_a, 2);

    getch(img_b, chY_b, 0);
    getch(img_b, chU_b, 1);
    getch(img_b, chV_b, 2);
}

void rewindFrame()
{
    prevFrame(img_a, reader_a);
    prevFrame(img_b, reader_b);
    diffYUV(img_a, img_b, diffY, 0);
    diffYUV(img_a, img_b, diffU, 1);
    diffYUV(img_a, img_b, diffV, 2);
    diffYUV(img_a, img_b, diff, 3);

    getch(img_a, chY_a, 0);
    getch(img_a, chU_a, 1);
    getch(img_a, chV_a, 2);

    getch(img_b, chY_b, 0);
    getch(img_b, chU_b, 1);
    getch(img_b, chV_b, 2);
}

int nextFrame()
{
    if(!grabFrame(img_a, reader_a)) return 1;
    if(!grabFrame(img_b, reader_b))
    {
        prevFrame(img_a, reader_a);
        return 1;
    }

    diffYUV(img_a, img_b, diffY, 0);
    diffYUV(img_a, img_b, diffU, 1);
    diffYUV(img_a, img_b, diffV, 2);
    diffYUV(img_a, img_b, diff, 3);

    getch(img_a, chY_a, 0);
    getch(img_a, chU_a, 1);
    getch(img_a, chV_a, 2);

    getch(img_b, chY_b, 0);
    getch(img_b, chU_b, 1);
    getch(img_b, chV_b, 2);
    return 0;
}

int getButton(int x, int y)
{
    int x_cnt;
    int y_cnt;
    int Xoffset = width;
    int Yoffset = ZOOM * 16;

    x_cnt = (x - Xoffset) / button_w;
    y_cnt = (y - Yoffset) / button_h;

    return 4 * y_cnt + x_cnt;
}



void handleEvents() 
{
    int btn;
    int x, y;
    SDL_Event event;
    Uint8 * keystates = SDL_GetKeyState(NULL);
    if(keystates[ SDLK_RIGHT])
    {
        nextFrame();
        update_overlay = 1;
    }
    if(keystates[ SDLK_LEFT])
    {
        rewindFrame();
        update_overlay = 1;
    }
    while( SDL_PollEvent( &event ) )
    {
        switch (event.type)
        {
            case SDL_KEYDOWN:
                switch (event.key.keysym.sym)
                {
                    case SDLK_ESCAPE:
                        end = 1;
                        break;
                        // T P
                    case SDLK_t:
                        grid = !grid;
                        update_overlay = 1;
                        break;
                    case SDLK_p:
                        pause = !pause;
                        break;
                        //H J K
                    case SDLK_h:
                        rewindFirstFrame();
                        update_overlay = 1;
                        break;
                    case SDLK_j:
                        rewindFrame();
                        update_overlay = 1;
                        break;
                    case SDLK_k:
                        nextFrame();
                        update_overlay = 1;
                        break;
                        // Q W E R
                    case SDLK_q:
                        shown = img_a;
                        update_overlay =1;
                        break;
                    case SDLK_w:
                        shown = chY_a;
                        update_overlay =1;
                        break;
                    case SDLK_e:
                        shown = chU_a;
                        update_overlay =1;
                        break;
                    case SDLK_r:
                        shown = chV_a;
                        update_overlay =1;
                        break;
                        //A S D F
                    case SDLK_a:
                        shown = img_b;
                        update_overlay =1;
                        break;
                    case SDLK_s:
                        shown = chY_b;
                        update_overlay =1;
                        break;
                    case SDLK_d:
                        shown = chU_b;
                        update_overlay =1;
                        break;
                    case SDLK_f:
                        shown = chV_b;
                        update_overlay =1;
                        break;
                        //Z X C V
                    case SDLK_z:
                        shown = diff;
                        update_overlay =1;
                        break;
                    case SDLK_x:
                        shown = diffY;
                        update_overlay =1;
                        break;
                    case SDLK_c:
                        shown = diffU;
                        update_overlay =1;
                        break;
                    case SDLK_v:
                        shown = diffV;
                        update_overlay =1;
                        break;
                    default:
                        break;
                }
                break;
            case SDL_MOUSEBUTTONDOWN:
                SDL_GetMouseState(&x, &y);
                if(x > width && y > ZOOM * 16)
                {
                    btn = getButton(x,y);
                    switch (btn)
                    {
                        // Q W E R
                        case 0:
                            shown = img_a;
                            update_overlay =1;
                            break;
                        case 1:
                            shown = chY_a;
                            update_overlay =1;
                            break;
                        case 2:
                            shown = chU_a;
                            update_overlay =1;
                            break;
                        case 3:
                            shown = chV_a;
                            update_overlay =1;
                            break;
                            //A S D F
                        case 4:
                            shown = img_b;
                            update_overlay =1;
                            break;
                        case 5:
                            shown = chY_b;
                            update_overlay =1;
                            break;
                        case 6:
                            shown = chU_b;
                            update_overlay =1;
                            break;
                        case 7:
                            shown = chV_b;
                            update_overlay =1;
                            break;
                            //Z X C V
                        case 8:
                            shown = diff;
                            update_overlay =1;
                            break;
                        case 9:
                            shown = diffY;
                            update_overlay =1;
                            break;
                        case 10:
                            shown = diffU;
                            update_overlay =1;
                            break;
                        case 11:
                            shown = diffV;
                            update_overlay =1;
                            break;
                    }
                }
                else
                {
                    update_mouse(x, y);
                    dumpBlock(x, y);
                }

                break;

        }
    }

}

void drawMacroblockEdges(SDL_Surface * surf, int w, int h)
{
    int i;
    for (i = 16; i < h; i+=16)
    {
        aalineRGBA(surf, 0, i, w, i, 255, 255, 255, 255);
    }
    for (i = 16; i < w; i+=16)
    {
        aalineRGBA(surf, i, 0, i, h, 255, 255, 255, 255);
    }
}



int main(int argc, const char *argv[])
{
    int i, j;
    int window_height;
    SDL_Rect rect;
    SDL_Surface *screen;
    SDL_Surface * mb;
    SDL_Surface * zoomed;

    SDL_Overlay * overlay = NULL;

    TTF_Font * font;
    SDL_Color font_color = {255, 255, 255};
    SDL_Surface * rendered_text;
    SDL_Rect font_location;

    /*FILE *writer;*/

    if (argc != 5 )
    {
        printf("cmd err\n");
        printf("Correct usage:\n");
        printf("\tyuvdiff <yuv file a> <yuv file b> <width> <height>\n");
        return 1;
    }

    width = atoi(argv[3]);
    height = atoi(argv[4]);

    /*button_h = (height - 256) / 3;*/
    button_h = 64;
    button_w = (ZOOM * 16) / 4;

    rect.x = 0;
    rect.y = 0;
    rect.w = width;
    rect.h = height;

    zoom_mb.x = 0;
    zoom_mb.y = 0;
    zoom_mb.w = 16;
    zoom_mb.h = 16;

    zoom_d.x = width;
    zoom_d.y = 0;
    zoom_d.w = ZOOM * 16;
    zoom_d.h = ZOOM * 16;

    reader_a = fopen(argv[1], "rb");
    reader_b = fopen(argv[2], "rb");

    if(reader_a == NULL || reader_b == NULL)
    {
        printf("fopen err\n");
        return 1;
    }

    SDL_Init (SDL_INIT_VIDEO);
    TTF_Init();

    font = TTF_OpenFont("FreeSans.ttf", 12);
    if(!font)
    {
        printf("can't open FreeSans.ttf\n");
        SDL_Quit();
        TTF_Quit();
        return 1;
    }

    window_height = (height > (button_h * 3) + (ZOOM * 16) + 64) ? height : (button_h * 3) + (ZOOM * 16) + 64;
    screen = SDL_SetVideoMode(width + ZOOM * 16, window_height, 0, 0);
    overlay = SDL_CreateYUVOverlay(width, height, SDL_IYUV_OVERLAY, screen);

    /*writer = fopen(argv[5], "w");*/

    img_a = initImgYUV(width, height);
    img_b = initImgYUV(width, height);

    chY_a = initImgYUV(width, height);
    chU_a = initImgYUV(width, height);
    chV_a = initImgYUV(width, height);

    chY_b = initImgYUV(width, height);
    chU_b = initImgYUV(width, height);
    chV_b = initImgYUV(width, height);

    diffY = initImgYUV(width, height);
    diffU = initImgYUV(width, height);
    diffV = initImgYUV(width, height);
    diff = initImgYUV(width, height);

    mb = SDL_CreateRGBSurface(0, 16, 16, 32, 0 ,0 ,0 ,0);

    shown = img_a;

    while (1)
    {

        if (!pause)
        {
            if(nextFrame())
                pause = 1;
            update_overlay = 1;
        }
        else
        {
            SDL_Delay(10);
        }

        handleEvents();
        if (end)
            break;


        convertToOverlay(shown, overlay);
        if(update_overlay)
        {
            SDL_DisplayYUVOverlay(overlay, &rect);
            update_overlay = 0;
        }

        SDL_BlitSurface(screen, &zoom_s, mb, &zoom_mb );
        zoomed = rotozoomSurface(mb, 0, ZOOM, 0);


        for (i = 0; i < 4; i++)
        {
            for (j = 0; j < 3; j++)
            {
                rendered_text = TTF_RenderText_Solid(font, button_text[3 * i + j], font_color);
                rectangleColor(screen,
                        width + i * button_w,
                        ZOOM * 16 + j     * button_h,
                        width + (i+1) * button_w - 1,
                        ZOOM * 16 + (j+1) * button_h - 1,
                        0xffffffff);


                font_location.x  =     width + i * button_w;
                font_location.y  =     ZOOM * 16 + j     * button_h;
                font_location.w  =     width + (i+1) * button_w - 1;
                font_location.h  =     ZOOM * 16 + (j+1) * button_h - 1;
                SDL_BlitSurface(rendered_text, NULL, screen, &font_location );
                SDL_FreeSurface(rendered_text);
            }
        }
        SDL_BlitSurface(zoomed, NULL, screen, &zoom_d );

        rendered_text = TTF_RenderText_Solid(font, "T: Toggle grid   P: Pause/Play", font_color);
        font_location.x  =     width + 16;
        font_location.y  =     ZOOM * 16 + button_h * 3 + 16;
        font_location.w  =     0;
        font_location.h  =     0;
        SDL_BlitSurface(rendered_text, NULL, screen, &font_location );
        SDL_FreeSurface(rendered_text);

        rendered_text = TTF_RenderText_Solid(font, "G: First Frame  Left/Right: Continuous Play", font_color);
        font_location.x  =     width + 16;
        font_location.y  =     ZOOM * 16 + button_h * 3 + 32;
        font_location.w  =     0;
        font_location.h  =     0;
        SDL_BlitSurface(rendered_text, NULL, screen, &font_location );
        SDL_FreeSurface(rendered_text);

        rendered_text = TTF_RenderText_Solid(font, "K: Next Frame  J: Previous Frame", font_color);
        font_location.x  =     width + 16;
        font_location.y  =     ZOOM * 16 + button_h * 3 + 48;
        font_location.w  =     0;
        font_location.h  =     0;
        SDL_BlitSurface(rendered_text, NULL, screen, &font_location );
        SDL_FreeSurface(rendered_text);

        if(grid)
            drawMacroblockEdges(screen, width, height);


        SDL_Flip(screen);
        SDL_FreeSurface(zoomed);
        zoomed = NULL;
        /*pause =1;*/
    }

    freeImgYUV(&img_a);
    freeImgYUV(&img_b);

    freeImgYUV(&chY_a);
    freeImgYUV(&chU_a);
    freeImgYUV(&chV_a);

    freeImgYUV(&chY_b);
    freeImgYUV(&chU_b);
    freeImgYUV(&chV_b);

    freeImgYUV(&diffY);
    freeImgYUV(&diffU);
    freeImgYUV(&diffV);
    freeImgYUV(&diff);

    fclose(reader_a);
    fclose(reader_b);
    /*fclose(writer);*/

    SDL_FreeYUVOverlay(overlay);
    SDL_FreeSurface(mb);

    TTF_CloseFont(font);
    SDL_Quit();

    return 0;
}
