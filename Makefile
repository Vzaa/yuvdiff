CC=gcc
CFLAGS=`sdl-config --cflags` 
LIBS=`sdl-config --libs` -lSDL_gfx -lSDL_ttf
OFLAGS=-O3 -Wall 

CFILES=imgyuv.c psnr.c yuvdiff.c 
HFILES=imgyuv.h psnr.h 
OFILES=imgyuv.o psnr.o

all: yuvdiff psnr_tool

psnr_tool: psnr_tool.c ${OFILES}
	${CC}    ${CFLAGS}  ${DEFS} -o psnr_tool psnr_tool.c ${OFILES} ${LIBS} ${OFLAGS}

yuvdiff: yuvdiff.c ${OFILES}
	${CC}    ${CFLAGS}   ${DEFS} -o yuvdiff yuvdiff.c ${OFILES} ${LIBS} ${OFLAGS}
psnr.o: psnr.c ${HFILES}
	${CC}   -c ${CFLAGS} ${DEFS} -o psnr.o psnr.c                 ${OFLAGS}
imgyuv.o: imgyuv.c ${HFILES}
	${CC}   -c ${CFLAGS} ${DEFS} -o imgyuv.o imgyuv.c             ${OFLAGS}

clean:
	rm *.o yuvdiff psnr_tool
