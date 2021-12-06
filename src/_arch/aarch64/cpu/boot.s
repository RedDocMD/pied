// Load address of symbol into register, PC-relative
// Symbol must lie within +/-4GiB of PC
.macro ADDR_REL register, symbol
	adrp \register, \symbol
	add  \register, \register, #:lo12:\symbol
.endm

// Load address of symbol into a register, absolute
.macro ADDR_ABS register, symbol
	movz \register, #:abs_g2:\symbol
	movk \register, #:abs_g1_nc:\symbol
	movk \register, #:abs_g0_nc:\symbol
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
	b.eq .L_relocate_binary
	stp xzr, xzr, [x0], #16
	b .L_bss_init_loop

	// Next, relocate the binary
.L_relocate_binary:
	ADDR_REL x0, __binary_nonzero_start // Address binary got loaded to
	ADDR_ABS x1, __binary_nonzero_start // Address binary got linked to
	ADDR_ABS x2, __binary_nonzero_end_exclusive

.L_copy_loop:
	ldr x3, [x0], #8
	str x3, [x1], #8
	cmp x1, x2
	b.lo .L_copy_loop

	// Prepare the jump to Rust code
	// Set the stack pointer
	ADDR_ABS x0, __boot_core_stack_end_exclusive
	mov sp, x0
	// Enter rust code
	ADDR_ABS x1, _start_rust
	br x1
	// Failsafe
	b .L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start