// Load address of symbol into register, PC-relative
// Symbol must lie within +/-4GiB of PC
.macro ADDR_REL register, symbol
	adrp \register, \symbol
	add  \register, \register, #:lo12:\symbol
.endm

.equ _core_id_mask, 0b11

.section .text._start

_start:
	// Infinitely wait for events (aka "park the core").

	// Read CPUID, stop other cores
	mrs x1, MPIDR_EL1
	and x1, x1, _core_id_mask
	ldr x2, BOOT_CORE_ID
	cmp x1, x2
	b.eq .L_Main_core
.L_parking_loop:
	wfe
	b	.L_parking_loop

.L_Main_core:
	// Clear bss
	ADDR_REL x0, __bss_start
	ADDR_REL x1, __bss_end_exclusive
.L_bss_init_loop:	
	cmp x0, x1
	b.eq .L_prepare_rust
	stp xzr, xzr, [x0], #16
	b .L_bss_init_loop

	// Prepare the jump to Rust code
.L_prepare_rust:
	// Set the stack pointer
	ADDR_REL x0, __boot_core_stack_end_exclusive
	mov sp, x0
	// Enter rust code
	bl _start_rust
	// Failsafe
	b .L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start