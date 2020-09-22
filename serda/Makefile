all:
	# gcc -o pub -g -I /usr/local/include -L /usr/local/lib -l ddsc publisher.c
	# gcc -o sub -g -I /usr/local/include -L /usr/local/lib -l ddsc subscriber.c
	# gcc -o lsub -g -I /usr/local/include -L /usr/local/lib -l ddsc loan-subscriber.c
	gcc -o bsub -g -I /usr/local/include -L /usr/local/lib -l ddsc blob-subscriber.c

clean:
	$(RM) -Rf *.dSYM
	$(RM) pub sub lsub
