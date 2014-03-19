CC=gcc
CFLAGS=`sdl-config --cflags`
LIBS=`sdl-config --libs` -lSDL_gfx -lSDL_ttf
OFLAGS=-O3 -Wall

CFILES=imgyuv.c yuvdiff.c
HFILES=imgyuv.h
OFILES=imgyuv.o

all: yuvdiff

yuvdiff: yuvdiff.c ${OFILES}
	${CC}    ${CFLAGS}   -o yuvdiff yuvdiff.c ${OFILES} ${LIBS} ${OFLAGS}

imgyuv.o: imgyuv.c ${HFILES}
	${CC}   -c ${CFLAGS} -o imgyuv.o imgyuv.c             ${OFLAGS}

clean:
	rm *.o yuvdiff
