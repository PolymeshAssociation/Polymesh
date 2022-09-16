;; Call chain extension by passing through input and output of this contract
(module
	(import "seal0" "seal_call_chain_extension"
		(func $seal_call_chain_extension (param i32 i32 i32 i32 i32) (result i32))
	)
	(import "seal0" "seal_input" (func $seal_input (param i32 i32)))
	(import "env" "memory" (memory 16 16))

	;; [0, 4) len of input output
	(data (i32.const 0) "10000")

	(func (export "deploy"))

	(func (export "call")
		;; Read input into memory at offset 4 and length at offset 0.
		(call $seal_input (i32.const 4) (i32.const 0))

		;; Input is expected to be `func_id` followed by arbitrary data.
		(call $seal_call_chain_extension
			;; func_id is first 4 bytes of input.
			(i32.load (i32.const 4))
			;; input_ptr, starts at offset 8 since func_id takes up 4 initial.
			(i32.const 8)
			;; input_len stored at 0 per above, but we need to discount func_id from length.
			(i32.sub (i32.load (i32.const 0)) (i32.const 4))
			(i32.const 0) ;; output_ptr -- ignored
			(i32.const 0) ;; output_len_ptr -- ignored
		)

		drop
	)
)
