BBV.close()
SWV.close()

while True:
	if WTPT > 210 * psi and BBV.is_open():
		BBV.close()
	elif WTPT < 190 * psi and BBV.is_closed():
		BBV.open()
	
	if KTPT < 90 * psi and SWV.is_closed():
		SWV.open()


