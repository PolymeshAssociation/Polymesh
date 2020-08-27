(module
  (type $t0 (func (result i32)))
  (type $t1 (func (param i32 i32 i32)))
  (type $t2 (func (param i32 i32)))
  (type $t3 (func (param i32) (result i32)))
  (type $t4 (func (param i32)))
  (type $t5 (func))
  (type $t6 (func (param i32)))
  (import "env" "ext_scratch_size" (func $env.ext_scratch_size (type $t0)))
  (import "env" "ext_scratch_read" (func $env.ext_scratch_read (type $t1)))
  (import "env" "ext_scratch_write" (func $env.ext_scratch_write (type $t2)))
  (import "env" "ext_get_storage" (func $env.ext_get_storage (type $t3)))
  (import "env" "ext_set_storage" (func $env.ext_set_storage (type $t1)))
  (import "env" "ext_clear_storage" (func $env.ext_clear_storage (type $t4)))
  (import "env" "memory" (memory $env.memory 2 16))
  (func $deploy (type $t0) (result i32)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32) (local $l6 i32)
    global.get $g0
    i32.const 48
    i32.sub
    local.tee $l0
    global.set $g0
    block $B0
      block $B1
        block $B2
          block $B3
            block $B4
              block $B5
                call $env.ext_scratch_size
                local.tee $l1
                i32.const 16385
                i32.lt_u
                if $I6
                  i32.const 65852
                  local.get $l1
                  i32.store
                  i32.const 65856
                  i32.const 0
                  local.get $l1
                  call $env.ext_scratch_read
                  i32.const 65852
                  i32.load
                  local.tee $l2
                  i32.const 16385
                  i32.ge_u
                  br_if $B5
                  local.get $l2
                  local.get $l1
                  i32.lt_u
                  br_if $B4
                  local.get $l0
                  i32.const 0
                  i32.store8 offset=20
                  local.get $l1
                  i32.eqz
                  br_if $B1
                  local.get $l0
                  i32.const 65856
                  i32.load8_u
                  i32.store8 offset=16
                  local.get $l0
                  i32.const 1
                  i32.store8 offset=20
                  local.get $l1
                  i32.const 1
                  i32.eq
                  br_if $B2
                  local.get $l0
                  i32.const 65857
                  i32.load8_u
                  i32.store8 offset=17
                  local.get $l0
                  i32.const 2
                  i32.store8 offset=20
                  local.get $l1
                  i32.const 2
                  i32.eq
                  br_if $B2
                  local.get $l0
                  i32.const 65858
                  i32.load8_u
                  i32.store8 offset=18
                  local.get $l0
                  i32.const 3
                  i32.store8 offset=20
                  local.get $l1
                  i32.const 3
                  i32.ne
                  br_if $B3
                  br $B2
                end
                unreachable
              end
              unreachable
            end
            unreachable
          end
          local.get $l0
          i32.const 65859
          i32.load8_u
          i32.store8 offset=19
          local.get $l0
          i32.load offset=16
          local.tee $l2
          i32.const 24
          i32.shr_u
          local.set $l4
          local.get $l2
          i32.const 16
          i32.shr_u
          local.set $l5
          local.get $l2
          i32.const 8
          i32.shr_u
          local.set $l6
          i32.const 2
          local.set $l3
          block $B7
            block $B8
              block $B9
                local.get $l2
                i32.const 255
                i32.and
                local.tee $l2
                i32.const 207
                i32.ne
                if $I10
                  local.get $l2
                  i32.const 65
                  i32.ne
                  br_if $B0
                  local.get $l1
                  i32.const -4
                  i32.add
                  i32.eqz
                  br_if $B0
                  local.get $l6
                  i32.const 255
                  i32.and
                  i32.const 230
                  i32.ne
                  br_if $B0
                  local.get $l5
                  i32.const 255
                  i32.and
                  i32.const 145
                  i32.ne
                  br_if $B0
                  local.get $l4
                  i32.const 252
                  i32.ne
                  br_if $B0
                  i32.const 0
                  local.set $l1
                  i32.const 65860
                  i32.load8_u
                  local.tee $l2
                  i32.const 1
                  i32.gt_u
                  br_if $B0
                  local.get $l2
                  i32.const 1
                  i32.sub
                  br_if $B8
                  br $B9
                end
                local.get $l6
                i32.const 255
                i32.and
                i32.const 238
                i32.ne
                br_if $B0
                local.get $l5
                i32.const 255
                i32.and
                i32.const 124
                i32.ne
                br_if $B0
                local.get $l4
                i32.const 8
                i32.ne
                br_if $B0
                i32.const 0
                call $f7
                local.get $l0
                i32.const 0
                i32.store8 offset=15
                br $B7
              end
              i32.const 1
              local.set $l1
            end
            i32.const 0
            call $f7
            local.get $l0
            local.get $l1
            i32.store8 offset=15
          end
          local.get $l0
          i32.const 40
          i32.add
          i64.const 0
          i64.store
          local.get $l0
          i32.const 32
          i32.add
          i64.const 0
          i64.store
          local.get $l0
          i32.const 24
          i32.add
          i64.const 0
          i64.store
          local.get $l0
          i64.const 0
          i64.store offset=16
          local.get $l0
          i32.const 15
          i32.add
          local.get $l0
          i32.const 16
          i32.add
          call $f8
          call $f9
          i32.const 3
          local.set $l3
          br $B0
        end
        local.get $l0
        i32.const 0
        i32.store8 offset=20
      end
      i32.const 2
      local.set $l3
    end
    local.get $l0
    i32.const 48
    i32.add
    global.set $g0
    local.get $l3
    i32.const 2
    i32.shl
    i32.const 65536
    i32.add
    i32.load)
  (func $f7 (type $t4) (param $p0 i32)
    (local $l1 i32) (local $l2 i32) (local $l3 i64) (local $l4 i64)
    global.get $g0
    i32.const 272
    i32.sub
    local.tee $l1
    global.set $g0
    i32.const 65568
    i64.load
    i64.const -2
    i64.add
    local.tee $l3
    i32.wrap_i64
    local.tee $l2
    i32.const 2
    local.get $l3
    i64.const 4
    i64.lt_u
    select
    i32.const -2
    i32.add
    i32.const 1
    i32.gt_u
    if $I0
      i64.const 3
      i64.const 2
      local.get $p0
      select
      local.set $l4
      block $B1
        local.get $l3
        i64.const 3
        i64.le_u
        if $I2
          local.get $l2
          i32.const 2
          i32.ne
          br_if $B1
        end
        call $f13
        block $B3
          i32.const 65568
          i64.load
          i64.const 1
          i64.ne
          br_if $B3
          i32.const 65608
          i32.load
          i32.const 2
          i32.eq
          br_if $B3
          local.get $l1
          i32.const 24
          i32.add
          i32.const 65600
          i64.load
          i64.store
          local.get $l1
          i32.const 16
          i32.add
          i32.const 65592
          i64.load
          i64.store
          local.get $l1
          i32.const 8
          i32.add
          i32.const 65584
          i64.load
          i64.store
          local.get $l1
          i64.const 1
          i64.store offset=32
          local.get $l1
          i32.const 65576
          i64.load
          i64.store
          local.get $l1
          call $env.ext_clear_storage
        end
        call $f14
        block $B4
          i32.const 65680
          i64.load
          i64.const 1
          i64.ne
          br_if $B4
          i32.const 65720
          i32.load
          i32.const 2
          i32.eq
          br_if $B4
          local.get $l1
          i32.const 24
          i32.add
          i32.const 65712
          i64.load
          i64.store
          local.get $l1
          i32.const 16
          i32.add
          i32.const 65704
          i64.load
          i64.store
          local.get $l1
          i32.const 8
          i32.add
          i32.const 65696
          i64.load
          i64.store
          local.get $l1
          i64.const 1
          i64.store offset=32
          local.get $l1
          i32.const 65688
          i64.load
          i64.store
          local.get $l1
          call $env.ext_clear_storage
        end
        call $f15
        block $B5
          i32.const 65736
          i64.load
          i64.const 1
          i64.ne
          br_if $B5
          i32.const 65776
          i32.load
          i32.const 2
          i32.eq
          br_if $B5
          local.get $l1
          i32.const 24
          i32.add
          i32.const 65768
          i64.load
          i64.store
          local.get $l1
          i32.const 16
          i32.add
          i32.const 65760
          i64.load
          i64.store
          local.get $l1
          i32.const 8
          i32.add
          i32.const 65752
          i64.load
          i64.store
          local.get $l1
          i64.const 1
          i64.store offset=32
          local.get $l1
          i32.const 65744
          i64.load
          i64.store
          local.get $l1
          call $env.ext_clear_storage
        end
        call $f16
      end
      i32.const 65568
      local.get $l4
      i64.store
      local.get $l1
      call $f20
      local.get $l1
      i32.const 272
      i32.add
      global.set $g0
      return
    end
    unreachable)
  (func $f8 (type $t2) (param $p0 i32) (param $p1 i32)
    (local $l2 i32)
    global.get $g0
    i32.const 48
    i32.sub
    local.tee $l2
    global.set $g0
    i32.const 65852
    i32.const 1
    i32.store
    i32.const 65856
    local.get $p0
    i32.load8_u
    i32.store8
    local.get $l2
    i32.const 32
    i32.add
    local.get $p1
    i32.const 24
    i32.add
    i64.load
    i64.store
    local.get $l2
    i32.const 24
    i32.add
    local.get $p1
    i32.const 16
    i32.add
    i64.load
    i64.store
    local.get $l2
    i32.const 16
    i32.add
    local.get $p1
    i32.const 8
    i32.add
    i64.load
    i64.store
    local.get $l2
    i64.const 1
    i64.store offset=40
    local.get $l2
    local.get $p1
    i64.load
    i64.store offset=8
    local.get $l2
    i32.const 8
    i32.add
    i32.const 65856
    i32.const 1
    call $env.ext_set_storage
    local.get $l2
    i32.const 48
    i32.add
    global.set $g0)
  (func $f9 (type $t5)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32) (local $l6 i32) (local $l7 i32) (local $l8 i32) (local $l9 i64) (local $l10 i64) (local $l11 i64) (local $l12 i64) (local $l13 i64) (local $l14 i64) (local $l15 i64) (local $l16 i64) (local $l17 i64)
    global.get $g0
    local.tee $l1
    local.get $l1
    i32.const 384
    i32.sub
    i32.const -32
    i32.and
    local.tee $l0
    global.set $g0
    i32.const 65568
    i64.load
    i64.const -2
    i64.add
    local.tee $l9
    i32.wrap_i64
    i32.const 2
    local.get $l9
    i64.const 4
    i64.lt_u
    select
    local.tee $l1
    i32.const 2
    i32.ge_u
    if $I0
      block $B1
        block $B2
          local.get $l1
          i32.const -3
          i32.add
          if $I3
            local.get $l0
            i32.const 24
            i32.add
            local.tee $l1
            i64.const -72340172838076674
            i64.store
            local.get $l0
            i32.const 16
            i32.add
            local.tee $l2
            i64.const -72340172838076674
            i64.store
            local.get $l0
            i32.const 8
            i32.add
            local.tee $l3
            i64.const -72340172838076674
            i64.store
            local.get $l0
            i64.const -72340172838076674
            i64.store
            i64.const 0
            local.set $l9
            local.get $l0
            i64.const 0
            i64.store offset=32
            i32.const 65568
            local.get $l0
            call $f17
            local.get $l3
            local.get $l3
            i64.load
            local.tee $l11
            local.get $l0
            i64.load
            local.tee $l10
            local.get $l0
            i64.load offset=32
            i64.add
            local.tee $l12
            local.get $l10
            i64.lt_u
            i64.extend_i32_u
            i64.add
            local.tee $l10
            i64.store
            local.get $l2
            local.get $l2
            i64.load
            local.tee $l13
            local.get $l10
            local.get $l11
            i64.lt_u
            i64.extend_i32_u
            i64.add
            local.tee $l11
            i64.store
            local.get $l1
            local.get $l1
            i64.load
            local.get $l11
            local.get $l13
            i64.lt_u
            i64.extend_i32_u
            i64.add
            local.tee $l13
            i64.store
            local.get $l0
            i64.const 4294967296
            i64.store offset=32
            local.get $l0
            local.get $l12
            i64.store
            block $B4 (result i32)
              i32.const 65664
              i32.load
              local.tee $l3
              i32.eqz
              if $I5
                i32.const 0
                local.set $l1
                i32.const 0
                local.set $l3
                i32.const 0
                br $B4
              end
              i32.const 65668
              i32.load
              local.set $l2
              local.get $l3
              local.set $l1
              loop $L6
                local.get $l9
                i64.const 4294967295
                i64.and
                local.get $l1
                i32.load16_u offset=6
                local.tee $l4
                i64.extend_i32_u
                i64.const 32
                i64.shl
                i64.or
                local.set $l9
                local.get $l2
                if $I7
                  local.get $l2
                  i32.const -1
                  i32.add
                  local.set $l2
                  local.get $l1
                  local.get $l4
                  i32.const 2
                  i32.shl
                  i32.add
                  i32.const 96
                  i32.add
                  i32.load
                  local.set $l1
                  local.get $l3
                  i32.load offset=96
                  local.set $l3
                  br $L6
                end
              end
              i32.const 65672
              i32.load
            end
            local.set $l4
            local.get $l0
            i32.const 312
            i32.add
            local.get $l9
            i64.store
            local.get $l0
            i32.const 308
            i32.add
            local.get $l1
            i32.store
            local.get $l0
            local.get $l4
            i32.store offset=320
            local.get $l0
            i32.const 0
            i32.store offset=304
            local.get $l0
            i64.const 0
            i64.store offset=296
            local.get $l0
            local.get $l3
            i32.store offset=292
            local.get $l0
            local.get $l2
            i32.store offset=288
            local.get $l4
            i32.eqz
            br_if $B1
            local.get $l0
            local.get $l4
            i32.const -1
            i32.add
            i32.store offset=320
            local.get $l0
            i32.const 288
            i32.add
            i32.const 0
            local.get $l3
            select
            local.tee $l5
            i32.load
            local.set $l3
            local.get $l5
            i32.load offset=8
            local.set $l6
            block $B8
              block $B9
                local.get $l5
                i32.load offset=12
                local.tee $l4
                local.get $l5
                i32.load offset=4
                local.tee $l1
                i32.load16_u offset=6
                i32.lt_u
                if $I10
                  local.get $l1
                  local.set $l2
                  br $B9
                end
                loop $L11
                  local.get $l1
                  i32.load
                  local.tee $l2
                  i32.eqz
                  br_if $B8
                  local.get $l3
                  i32.const 1
                  i32.add
                  local.set $l3
                  local.get $l1
                  i32.load16_u offset=4
                  local.tee $l4
                  local.get $l2
                  local.tee $l1
                  i32.load16_u offset=6
                  i32.ge_u
                  br_if $L11
                end
              end
              local.get $l6
              i64.extend_i32_u
              local.get $l4
              i64.extend_i32_u
              i64.const 32
              i64.shl
              i64.or
              local.set $l9
              br $B2
            end
            local.get $l6
            i64.extend_i32_u
            local.set $l9
            i32.const 0
            local.set $l2
            br $B2
          end
          unreachable
        end
        local.get $l9
        i64.const 32
        i64.shr_u
        i32.wrap_i64
        local.tee $l6
        i32.const 1
        i32.add
        local.set $l4
        local.get $l9
        i32.wrap_i64
        local.set $l7
        block $B12
          local.get $l3
          i32.eqz
          if $I13
            local.get $l2
            local.set $l1
            br $B12
          end
          local.get $l2
          local.get $l4
          i32.const 2
          i32.shl
          i32.add
          i32.const 96
          i32.add
          i32.load
          local.set $l1
          i32.const 0
          local.set $l4
          local.get $l3
          i32.const -1
          i32.add
          local.tee $l3
          i32.eqz
          br_if $B12
          loop $L14
            local.get $l1
            i32.load offset=96
            local.set $l1
            local.get $l3
            i32.const -1
            i32.add
            local.tee $l3
            br_if $L14
          end
        end
        local.get $l5
        local.get $l4
        i32.store offset=12
        local.get $l5
        local.get $l7
        i32.store offset=8
        local.get $l5
        local.get $l1
        i32.store offset=4
        local.get $l5
        i32.const 0
        i32.store
        local.get $l2
        local.get $l6
        i32.const 2
        i32.shl
        i32.add
        local.tee $l2
        i32.const 52
        i32.add
        local.set $l1
        local.get $l2
        i32.const 8
        i32.add
        local.set $l2
        loop $L15
          local.get $l0
          local.get $l12
          local.get $l2
          i64.load32_u
          i64.add
          local.tee $l9
          i64.store offset=352
          local.get $l0
          local.get $l10
          local.get $l9
          local.get $l12
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l9
          i64.store offset=360
          local.get $l0
          local.get $l11
          local.get $l9
          local.get $l10
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l9
          i64.store offset=368
          local.get $l0
          local.get $l13
          local.get $l9
          local.get $l11
          i64.lt_u
          i64.extend_i32_u
          i64.add
          i64.store offset=376
          local.get $l1
          i32.load
          local.tee $l1
          i32.load8_u offset=40
          local.set $l2
          local.get $l1
          i32.const 1
          i32.store8 offset=40
          block $B16
            local.get $l2
            i32.const 1
            i32.and
            br_if $B16
            local.get $l1
            i32.const 1
            i32.store8 offset=40
            local.get $l1
            i32.load
            i32.const 1
            i32.ne
            if $I17
              local.get $l0
              i32.const 352
              i32.add
              call $env.ext_clear_storage
              br $B16
            end
            i32.const 65880
            local.get $l1
            i32.const 32
            i32.add
            i64.load align=1
            i64.store align=4
            i32.const 65872
            local.get $l1
            i32.const 24
            i32.add
            i64.load align=1
            i64.store align=4
            i32.const 65864
            local.get $l1
            i32.const 16
            i32.add
            i64.load align=1
            i64.store align=4
            i32.const 65856
            local.get $l1
            i32.const 8
            i32.add
            i64.load align=1
            i64.store align=4
            i32.const 65852
            i32.const 36
            i32.store
            i32.const 65888
            local.get $l1
            i32.load offset=4
            i32.store
            local.get $l0
            i32.const 352
            i32.add
            i32.const 65856
            i32.const 36
            call $env.ext_set_storage
          end
          local.get $l0
          i32.load offset=320
          local.tee $l1
          i32.eqz
          br_if $B1
          local.get $l0
          local.get $l1
          i32.const -1
          i32.add
          i32.store offset=320
          local.get $l0
          i32.const 288
          i32.add
          i32.const 0
          local.get $l0
          i32.load offset=292
          select
          local.tee $l5
          i32.load
          local.set $l3
          local.get $l5
          i32.load offset=8
          local.set $l6
          block $B18 (result i64)
            block $B19
              local.get $l5
              i32.load offset=12
              local.tee $l4
              local.get $l5
              i32.load offset=4
              local.tee $l1
              i32.load16_u offset=6
              i32.lt_u
              if $I20
                local.get $l1
                local.set $l2
                br $B19
              end
              loop $L21
                local.get $l1
                i32.load
                local.tee $l2
                if $I22
                  local.get $l3
                  i32.const 1
                  i32.add
                  local.set $l3
                  local.get $l1
                  i32.load16_u offset=4
                  local.tee $l4
                  local.get $l2
                  local.tee $l1
                  i32.load16_u offset=6
                  i32.ge_u
                  br_if $L21
                  br $B19
                end
              end
              i32.const 0
              local.set $l2
              local.get $l6
              i64.extend_i32_u
              br $B18
            end
            local.get $l6
            i64.extend_i32_u
            local.get $l4
            i64.extend_i32_u
            i64.const 32
            i64.shl
            i64.or
          end
          local.tee $l9
          i64.const 32
          i64.shr_u
          i32.wrap_i64
          local.tee $l6
          i32.const 1
          i32.add
          local.set $l4
          local.get $l9
          i32.wrap_i64
          local.set $l7
          block $B23
            local.get $l3
            i32.eqz
            if $I24
              local.get $l2
              local.set $l1
              br $B23
            end
            local.get $l2
            local.get $l4
            i32.const 2
            i32.shl
            i32.add
            i32.const 96
            i32.add
            i32.load
            local.set $l1
            i32.const 0
            local.set $l4
            local.get $l3
            i32.const -1
            i32.add
            local.tee $l3
            i32.eqz
            br_if $B23
            loop $L25
              local.get $l1
              i32.load offset=96
              local.set $l1
              local.get $l3
              i32.const -1
              i32.add
              local.tee $l3
              br_if $L25
            end
          end
          local.get $l5
          local.get $l4
          i32.store offset=12
          local.get $l5
          local.get $l7
          i32.store offset=8
          local.get $l5
          local.get $l1
          i32.store offset=4
          local.get $l5
          i32.const 0
          i32.store
          local.get $l2
          local.get $l6
          i32.const 2
          i32.shl
          i32.add
          local.tee $l2
          i32.const 52
          i32.add
          local.set $l1
          local.get $l2
          i32.const 8
          i32.add
          local.set $l2
          br $L15
        end
        unreachable
      end
      i32.const 65680
      local.get $l0
      call $f17
      i32.const 65736
      local.get $l0
      call $f17
      local.get $l0
      i64.load offset=32
      local.set $l9
      local.get $l0
      i64.const 4294967296
      i64.store offset=32
      local.get $l0
      local.get $l9
      local.get $l0
      i64.load
      local.tee $l10
      i64.add
      local.tee $l12
      i64.store
      local.get $l0
      local.get $l0
      i64.load offset=8
      local.tee $l9
      local.get $l12
      local.get $l10
      i64.lt_u
      i64.extend_i32_u
      i64.add
      local.tee $l10
      i64.store offset=8
      local.get $l0
      local.get $l0
      i64.load offset=16
      local.tee $l13
      local.get $l10
      local.get $l9
      i64.lt_u
      i64.extend_i32_u
      i64.add
      local.tee $l11
      i64.store offset=16
      local.get $l0
      local.get $l0
      i64.load offset=24
      local.get $l11
      local.get $l13
      i64.lt_u
      i64.extend_i32_u
      i64.add
      local.tee $l13
      i64.store offset=24
      block $B26 (result i32)
        i32.const 65832
        i32.load
        local.tee $l3
        i32.eqz
        if $I27
          i32.const 0
          local.set $l3
          i32.const 0
          local.set $l1
          i32.const 0
          br $B26
        end
        i32.const 65836
        i32.load
        local.set $l2
        i64.const 0
        local.set $l9
        local.get $l3
        local.set $l1
        loop $L28
          local.get $l9
          i64.const 4294967295
          i64.and
          local.get $l1
          i32.load16_u offset=6
          local.tee $l4
          i64.extend_i32_u
          i64.const 32
          i64.shl
          i64.or
          local.set $l9
          local.get $l2
          if $I29
            local.get $l2
            i32.const -1
            i32.add
            local.set $l2
            local.get $l1
            local.get $l4
            i32.const 2
            i32.shl
            i32.add
            i32.const 96
            i32.add
            i32.load
            local.set $l1
            local.get $l3
            i32.load offset=96
            local.set $l3
            br $L28
          end
        end
        i32.const 65840
        i32.load
      end
      local.set $l4
      local.get $l0
      i32.const 312
      i32.add
      local.get $l9
      i64.store
      local.get $l0
      i32.const 308
      i32.add
      local.get $l1
      i32.store
      local.get $l0
      local.get $l4
      i32.store offset=320
      local.get $l0
      i32.const 0
      i32.store offset=304
      local.get $l0
      i64.const 0
      i64.store offset=296
      local.get $l0
      local.get $l3
      i32.store offset=292
      local.get $l0
      local.get $l2
      i32.store offset=288
      block $B30
        local.get $l4
        i32.eqz
        br_if $B30
        local.get $l0
        local.get $l4
        i32.const -1
        i32.add
        i32.store offset=320
        local.get $l0
        i32.const 288
        i32.add
        i32.const 0
        local.get $l3
        select
        local.tee $l5
        i32.load
        local.set $l3
        local.get $l5
        i32.load offset=8
        local.set $l6
        block $B31 (result i64)
          block $B32
            block $B33
              local.get $l5
              i32.load offset=12
              local.tee $l4
              local.get $l5
              i32.load offset=4
              local.tee $l1
              i32.load16_u offset=6
              i32.lt_u
              if $I34
                local.get $l1
                local.set $l2
                br $B33
              end
              loop $L35
                local.get $l1
                i32.load
                local.tee $l2
                i32.eqz
                br_if $B32
                local.get $l3
                i32.const 1
                i32.add
                local.set $l3
                local.get $l1
                i32.load16_u offset=4
                local.tee $l4
                local.get $l2
                local.tee $l1
                i32.load16_u offset=6
                i32.ge_u
                br_if $L35
              end
            end
            local.get $l6
            i64.extend_i32_u
            local.get $l4
            i64.extend_i32_u
            i64.const 32
            i64.shl
            i64.or
            br $B31
          end
          i32.const 0
          local.set $l2
          local.get $l6
          i64.extend_i32_u
        end
        local.tee $l9
        i64.const 32
        i64.shr_u
        i32.wrap_i64
        local.tee $l6
        i32.const 1
        i32.add
        local.set $l4
        local.get $l9
        i32.wrap_i64
        local.set $l7
        block $B36
          local.get $l3
          i32.eqz
          if $I37
            local.get $l2
            local.set $l1
            br $B36
          end
          local.get $l2
          local.get $l4
          i32.const 2
          i32.shl
          i32.add
          i32.const 96
          i32.add
          i32.load
          local.set $l1
          i32.const 0
          local.set $l4
          local.get $l3
          i32.const -1
          i32.add
          local.tee $l3
          i32.eqz
          br_if $B36
          loop $L38
            local.get $l1
            i32.load offset=96
            local.set $l1
            local.get $l3
            i32.const -1
            i32.add
            local.tee $l3
            br_if $L38
          end
        end
        local.get $l5
        local.get $l4
        i32.store offset=12
        local.get $l5
        local.get $l7
        i32.store offset=8
        local.get $l5
        local.get $l1
        i32.store offset=4
        local.get $l5
        i32.const 0
        i32.store
        local.get $l2
        local.get $l6
        i32.const 2
        i32.shl
        i32.add
        local.tee $l2
        i32.const 52
        i32.add
        local.set $l1
        local.get $l2
        i32.const 8
        i32.add
        local.set $l2
        loop $L39
          local.get $l0
          local.get $l12
          local.get $l2
          i64.load32_u
          i64.add
          local.tee $l9
          i64.store offset=352
          local.get $l0
          local.get $l10
          local.get $l9
          local.get $l12
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l9
          i64.store offset=360
          local.get $l0
          local.get $l11
          local.get $l9
          local.get $l10
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l9
          i64.store offset=368
          local.get $l0
          local.get $l13
          local.get $l9
          local.get $l11
          i64.lt_u
          i64.extend_i32_u
          i64.add
          i64.store offset=376
          local.get $l1
          i32.load
          local.tee $l1
          i32.load8_u offset=40
          local.set $l2
          local.get $l1
          i32.const 1
          i32.store8 offset=40
          block $B40
            local.get $l2
            i32.const 1
            i32.and
            br_if $B40
            local.get $l1
            i32.const 1
            i32.store8 offset=40
            local.get $l1
            i64.load
            i64.const 1
            i64.ne
            if $I41
              local.get $l0
              i32.const 352
              i32.add
              call $env.ext_clear_storage
              br $B40
            end
            i32.const 65852
            i32.const 32
            i32.store
            i32.const 65856
            local.get $l1
            i64.load offset=8 align=1
            i64.store align=4
            i32.const 65880
            local.get $l1
            i32.const 32
            i32.add
            i64.load align=1
            i64.store align=4
            i32.const 65872
            local.get $l1
            i32.const 24
            i32.add
            i64.load align=1
            i64.store align=4
            i32.const 65864
            local.get $l1
            i32.const 16
            i32.add
            i64.load align=1
            i64.store align=4
            local.get $l0
            i32.const 352
            i32.add
            i32.const 65856
            i32.const 32
            call $env.ext_set_storage
          end
          local.get $l0
          i32.load offset=320
          local.tee $l1
          i32.eqz
          br_if $B30
          local.get $l0
          local.get $l1
          i32.const -1
          i32.add
          i32.store offset=320
          local.get $l0
          i32.const 288
          i32.add
          i32.const 0
          local.get $l0
          i32.load offset=292
          select
          local.tee $l5
          i32.load
          local.set $l3
          local.get $l5
          i32.load offset=8
          local.set $l6
          block $B42 (result i64)
            block $B43
              local.get $l5
              i32.load offset=12
              local.tee $l4
              local.get $l5
              i32.load offset=4
              local.tee $l1
              i32.load16_u offset=6
              i32.lt_u
              if $I44
                local.get $l1
                local.set $l2
                br $B43
              end
              loop $L45
                local.get $l1
                i32.load
                local.tee $l2
                if $I46
                  local.get $l3
                  i32.const 1
                  i32.add
                  local.set $l3
                  local.get $l1
                  i32.load16_u offset=4
                  local.tee $l4
                  local.get $l2
                  local.tee $l1
                  i32.load16_u offset=6
                  i32.ge_u
                  br_if $L45
                  br $B43
                end
              end
              i32.const 0
              local.set $l2
              local.get $l6
              i64.extend_i32_u
              br $B42
            end
            local.get $l6
            i64.extend_i32_u
            local.get $l4
            i64.extend_i32_u
            i64.const 32
            i64.shl
            i64.or
          end
          local.tee $l9
          i64.const 32
          i64.shr_u
          i32.wrap_i64
          local.tee $l6
          i32.const 1
          i32.add
          local.set $l4
          local.get $l9
          i32.wrap_i64
          local.set $l7
          block $B47
            local.get $l3
            i32.eqz
            if $I48
              local.get $l2
              local.set $l1
              br $B47
            end
            local.get $l2
            local.get $l4
            i32.const 2
            i32.shl
            i32.add
            i32.const 96
            i32.add
            i32.load
            local.set $l1
            i32.const 0
            local.set $l4
            local.get $l3
            i32.const -1
            i32.add
            local.tee $l3
            i32.eqz
            br_if $B47
            loop $L49
              local.get $l1
              i32.load offset=96
              local.set $l1
              local.get $l3
              i32.const -1
              i32.add
              local.tee $l3
              br_if $L49
            end
          end
          local.get $l5
          local.get $l4
          i32.store offset=12
          local.get $l5
          local.get $l7
          i32.store offset=8
          local.get $l5
          local.get $l1
          i32.store offset=4
          local.get $l5
          i32.const 0
          i32.store
          local.get $l2
          local.get $l6
          i32.const 2
          i32.shl
          i32.add
          local.tee $l2
          i32.const 52
          i32.add
          local.set $l1
          local.get $l2
          i32.const 8
          i32.add
          local.set $l2
          br $L39
        end
        unreachable
      end
      local.get $l0
      i32.const 0
      i32.store8 offset=368
      local.get $l0
      i64.const 1
      i64.store offset=360
      local.get $l0
      i32.const 0
      i32.store
      local.get $l0
      i32.const 0
      i32.store offset=8
      local.get $l0
      i64.const 0
      i64.store offset=376
      local.get $l0
      i64.load
      local.set $l9
      local.get $l0
      i64.load offset=8
      local.set $l12
      local.get $l0
      i32.const 0
      i32.store offset=288
      local.get $l0
      i32.const 0
      i32.store8 offset=24
      local.get $l0
      i64.load
      local.set $l10
      local.get $l0
      i64.load offset=8
      local.set $l11
      local.get $l0
      i64.load offset=288
      local.set $l13
      local.get $l0
      i64.load offset=304
      local.set $l14
      local.get $l0
      i64.load offset=312
      local.set $l15
      local.get $l0
      i64.load offset=352
      local.set $l16
      local.get $l0
      i64.load offset=368
      local.set $l17
      i32.const 65784
      local.get $l0
      i64.load offset=24
      i64.store
      i32.const 65776
      i64.const 1
      i64.store
      i32.const 65768
      local.get $l11
      i64.store
      i32.const 65760
      local.get $l10
      i64.store
      i32.const 65752
      local.get $l15
      i64.store
      i32.const 65744
      local.get $l14
      i64.store
      i32.const 65736
      i64.const 0
      i64.store
      i32.const 65728
      local.get $l13
      i64.store
      i32.const 65672
      local.get $l12
      i64.store
      i32.const 65664
      local.get $l9
      i64.store
      i32.const 65680
      i64.const 0
      i64.store
      i32.const 65624
      i64.const 0
      i64.store
      i32.const 65616
      local.get $l17
      i64.store
      i32.const 65608
      i64.const 1
      i64.store
      i32.const 65600
      local.get $l16
      i64.store
      i32.const 65792
      i64.const 0
      i64.store
      i32.const 65720
      i64.const 1
      i64.store
      i32.const 65568
      i64.const 0
      i64.store
      i32.const 65840
      i32.const 0
      i32.store
      i32.const 65832
      i32.const 0
      i32.store
      call $f13
      block $B50
        i32.const 65568
        i64.load
        i64.const 1
        i64.ne
        br_if $B50
        i32.const 65608
        i32.load
        i32.const 2
        i32.eq
        br_if $B50
        local.get $l0
        i32.const 24
        i32.add
        i32.const 65600
        i64.load
        i64.store
        local.get $l0
        i32.const 16
        i32.add
        i32.const 65592
        i64.load
        i64.store
        local.get $l0
        i32.const 8
        i32.add
        i32.const 65584
        i64.load
        i64.store
        local.get $l0
        i64.const 1
        i64.store offset=32
        local.get $l0
        i32.const 65576
        i64.load
        i64.store
        local.get $l0
        call $env.ext_clear_storage
      end
      call $f14
      block $B51
        i32.const 65680
        i64.load
        i64.const 1
        i64.ne
        br_if $B51
        i32.const 65720
        i32.load
        i32.const 2
        i32.eq
        br_if $B51
        local.get $l0
        i32.const 24
        i32.add
        i32.const 65712
        i64.load
        i64.store
        local.get $l0
        i32.const 16
        i32.add
        i32.const 65704
        i64.load
        i64.store
        local.get $l0
        i32.const 8
        i32.add
        i32.const 65696
        i64.load
        i64.store
        local.get $l0
        i64.const 1
        i64.store offset=32
        local.get $l0
        i32.const 65688
        i64.load
        i64.store
        local.get $l0
        call $env.ext_clear_storage
      end
      call $f15
      block $B52
        i32.const 65736
        i64.load
        i64.const 1
        i64.ne
        br_if $B52
        i32.const 65776
        i32.load
        i32.const 2
        i32.eq
        br_if $B52
        local.get $l0
        i32.const 24
        i32.add
        i32.const 65768
        i64.load
        i64.store
        local.get $l0
        i32.const 16
        i32.add
        i32.const 65760
        i64.load
        i64.store
        local.get $l0
        i32.const 8
        i32.add
        i32.const 65752
        i64.load
        i64.store
        local.get $l0
        i64.const 1
        i64.store offset=32
        local.get $l0
        i32.const 65744
        i64.load
        i64.store
        local.get $l0
        call $env.ext_clear_storage
      end
      call $f16
      i32.const 65568
      i64.const 5
      i64.store
      local.get $l0
      call $f20
    end
    global.set $g0)
  (func $call (type $t0) (result i32)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32)
    global.get $g0
    i32.const 48
    i32.sub
    local.tee $l0
    global.set $g0
    block $B0
      block $B1
        block $B2
          block $B3
            block $B4
              block $B5
                call $env.ext_scratch_size
                local.tee $l1
                i32.const 16385
                i32.lt_u
                if $I6
                  i32.const 65852
                  local.get $l1
                  i32.store
                  i32.const 65856
                  i32.const 0
                  local.get $l1
                  call $env.ext_scratch_read
                  i32.const 65852
                  i32.load
                  local.tee $l2
                  i32.const 16385
                  i32.ge_u
                  br_if $B5
                  local.get $l2
                  local.get $l1
                  i32.lt_u
                  br_if $B4
                  local.get $l0
                  i32.const 0
                  i32.store8 offset=12
                  local.get $l1
                  i32.eqz
                  br_if $B1
                  local.get $l0
                  i32.const 65856
                  i32.load8_u
                  i32.store8 offset=8
                  local.get $l0
                  i32.const 1
                  i32.store8 offset=12
                  local.get $l1
                  i32.const 1
                  i32.eq
                  br_if $B2
                  local.get $l0
                  i32.const 65857
                  i32.load8_u
                  i32.store8 offset=9
                  local.get $l0
                  i32.const 2
                  i32.store8 offset=12
                  local.get $l1
                  i32.const 2
                  i32.eq
                  br_if $B2
                  local.get $l0
                  i32.const 65858
                  i32.load8_u
                  i32.store8 offset=10
                  local.get $l0
                  i32.const 3
                  i32.store8 offset=12
                  local.get $l1
                  i32.const 3
                  i32.ne
                  br_if $B3
                  br $B2
                end
                unreachable
              end
              unreachable
            end
            unreachable
          end
          local.get $l0
          i32.const 65859
          i32.load8_u
          i32.store8 offset=11
          local.get $l0
          i32.load offset=8
          local.tee $l1
          i32.const 24
          i32.shr_u
          local.set $l3
          local.get $l1
          i32.const 16
          i32.shr_u
          local.set $l4
          local.get $l1
          i32.const 8
          i32.shr_u
          local.set $l5
          i32.const 2
          local.set $l2
          block $B7
            local.get $l1
            i32.const 255
            i32.and
            local.tee $l1
            i32.const 104
            i32.ne
            if $I8
              local.get $l1
              i32.const 217
              i32.ne
              br_if $B0
              local.get $l5
              i32.const 255
              i32.and
              i32.const 110
              i32.ne
              br_if $B0
              local.get $l4
              i32.const 255
              i32.and
              i32.const 249
              i32.ne
              br_if $B0
              local.get $l3
              i32.const 81
              i32.ne
              br_if $B0
              i32.const 1
              call $f7
              local.get $l0
              i32.const 32
              i32.add
              i64.const 0
              i64.store
              local.get $l0
              i32.const 24
              i32.add
              i64.const 0
              i64.store
              local.get $l0
              i32.const 16
              i32.add
              i64.const 0
              i64.store
              local.get $l0
              i64.const 0
              i64.store offset=8
              local.get $l0
              local.get $l0
              i32.const 8
              i32.add
              call $f11
              i32.const 1
              i32.xor
              i32.store8 offset=47
              local.get $l0
              i32.const 47
              i32.add
              local.get $l0
              i32.const 8
              i32.add
              call $f8
              call $f9
              br $B7
            end
            local.get $l5
            i32.const 255
            i32.and
            i32.const 23
            i32.ne
            br_if $B0
            local.get $l4
            i32.const 255
            i32.and
            i32.const 192
            i32.ne
            br_if $B0
            local.get $l3
            i32.const 15
            i32.ne
            br_if $B0
            i32.const 1
            call $f7
            local.get $l0
            i32.const 32
            i32.add
            i64.const 0
            i64.store
            local.get $l0
            i32.const 24
            i32.add
            i64.const 0
            i64.store
            local.get $l0
            i32.const 16
            i32.add
            i64.const 0
            i64.store
            local.get $l0
            i64.const 0
            i64.store offset=8
            local.get $l0
            i32.const 8
            i32.add
            call $f11
            local.set $l1
            call $f9
            i32.const 65852
            i32.const 1
            i32.store
            i32.const 65856
            local.get $l1
            i32.store8
            i32.const 65856
            i32.const 1
            call $env.ext_scratch_write
          end
          i32.const 3
          local.set $l2
          br $B0
        end
        local.get $l0
        i32.const 0
        i32.store8 offset=12
      end
      i32.const 2
      local.set $l2
    end
    local.get $l0
    i32.const 48
    i32.add
    global.set $g0
    local.get $l2
    i32.const 2
    i32.shl
    i32.const 65536
    i32.add
    i32.load)
  (func $f11 (type $t3) (param $p0 i32) (result i32)
    (local $l1 i32) (local $l2 i32)
    global.get $g0
    i32.const 48
    i32.sub
    local.tee $l1
    global.set $g0
    local.get $l1
    i32.const 32
    i32.add
    local.get $p0
    i32.const 24
    i32.add
    i64.load
    i64.store
    local.get $l1
    i32.const 24
    i32.add
    local.get $p0
    i32.const 16
    i32.add
    i64.load
    i64.store
    local.get $l1
    i32.const 16
    i32.add
    local.get $p0
    i32.const 8
    i32.add
    i64.load
    i64.store
    local.get $l1
    i64.const 1
    i64.store offset=40
    local.get $l1
    local.get $p0
    i64.load
    i64.store offset=8
    block $B0
      block $B1
        block $B2
          block $B3
            block $B4
              block $B5
                block $B6
                  local.get $l1
                  i32.const 8
                  i32.add
                  call $env.ext_get_storage
                  local.tee $p0
                  if $I7
                    local.get $p0
                    i32.const 1
                    i32.eq
                    br_if $B6
                    br $B0
                  end
                  call $env.ext_scratch_size
                  local.tee $p0
                  i32.const 16385
                  i32.ge_u
                  br_if $B0
                  i32.const 65852
                  local.get $p0
                  i32.store
                  i32.const 65856
                  i32.const 0
                  local.get $p0
                  call $env.ext_scratch_read
                  i32.const 65852
                  i32.load
                  local.tee $l2
                  i32.const 16385
                  i32.ge_u
                  br_if $B2
                  local.get $l2
                  local.get $p0
                  i32.lt_u
                  br_if $B1
                  local.get $p0
                  i32.eqz
                  br_if $B5
                  i32.const 0
                  local.set $p0
                  i32.const 65856
                  i32.load8_u
                  local.tee $l2
                  i32.const 1
                  i32.gt_u
                  br_if $B5
                  local.get $l2
                  i32.const 1
                  i32.sub
                  br_if $B3
                  br $B4
                end
                unreachable
              end
              unreachable
            end
            i32.const 256
            local.set $p0
          end
          local.get $l1
          i32.const 48
          i32.add
          global.set $g0
          local.get $p0
          i32.const 0
          i32.ne
          return
        end
        unreachable
      end
      unreachable
    end
    unreachable)
  (func $f12 (type $t2) (param $p0 i32) (param $p1 i32)
    (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32) (local $l6 i32) (local $l7 i32)
    block $B0
      local.get $p0
      i32.eqz
      br_if $B0
      local.get $p1
      i32.eqz
      br_if $B0
      i32.const 65848
      i32.load
      local.set $l4
      local.get $p0
      i32.const 0
      i32.store
      local.get $p0
      i32.const -8
      i32.add
      local.tee $p1
      local.get $p1
      i32.load
      local.tee $l3
      i32.const -2
      i32.and
      i32.store
      block $B1
        block $B2
          block $B3
            local.get $p0
            i32.const -4
            i32.add
            local.tee $l6
            i32.load
            i32.const -4
            i32.and
            local.tee $l2
            if $I4
              local.get $l2
              i32.load
              local.tee $l7
              i32.const 1
              i32.and
              i32.eqz
              br_if $B3
            end
            local.get $l3
            i32.const -4
            i32.and
            local.tee $l2
            i32.eqz
            br_if $B2
            i32.const 0
            local.get $l2
            local.get $l3
            i32.const 2
            i32.and
            select
            local.tee $l2
            i32.eqz
            br_if $B2
            local.get $l2
            i32.load8_u
            i32.const 1
            i32.and
            br_if $B2
            local.get $p0
            local.get $l2
            i32.load offset=8
            i32.const -4
            i32.and
            i32.store
            local.get $l2
            local.get $p1
            i32.const 1
            i32.or
            i32.store offset=8
            local.get $l4
            local.set $p1
            br $B1
          end
          block $B5
            block $B6
              local.get $l3
              i32.const -4
              i32.and
              local.tee $p0
              i32.eqz
              if $I7
                local.get $l2
                local.set $l5
                br $B6
              end
              local.get $l2
              local.set $l5
              i32.const 0
              local.get $p0
              local.get $l3
              i32.const 2
              i32.and
              select
              local.tee $l3
              i32.eqz
              br_if $B6
              local.get $l3
              local.get $l3
              i32.load offset=4
              i32.const 3
              i32.and
              local.get $l2
              i32.or
              i32.store offset=4
              local.get $l6
              i32.load
              local.tee $p0
              i32.const -4
              i32.and
              local.tee $l5
              i32.eqz
              br_if $B5
              local.get $p1
              i32.load
              i32.const -4
              i32.and
              local.set $p0
              local.get $l5
              i32.load
              local.set $l7
            end
            local.get $l5
            local.get $l7
            i32.const 3
            i32.and
            local.get $p0
            i32.or
            i32.store
            local.get $l6
            i32.load
            local.set $p0
          end
          local.get $l6
          local.get $p0
          i32.const 3
          i32.and
          i32.store
          local.get $p1
          local.get $p1
          i32.load
          local.tee $p0
          i32.const 3
          i32.and
          i32.store
          local.get $p0
          i32.const 2
          i32.and
          i32.eqz
          if $I8
            local.get $l4
            local.set $p1
            br $B1
          end
          local.get $l2
          local.get $l2
          i32.load
          i32.const 2
          i32.or
          i32.store
          local.get $l4
          local.set $p1
          br $B1
        end
        local.get $p0
        local.get $l4
        i32.store
      end
      i32.const 65848
      local.get $p1
      i32.store
    end)
  (func $f13 (type $t5)
    (local $l0 i32) (local $l1 i32) (local $l2 i64) (local $l3 i64) (local $l4 i64) (local $l5 i64) (local $l6 i64)
    global.get $g0
    i32.const 32
    i32.sub
    local.tee $l0
    global.set $g0
    block $B0
      block $B1
        i32.const 65624
        i64.load
        i64.const 1
        i64.ne
        br_if $B1
        i32.const 65568
        call $f18
        i32.load
        local.tee $l1
        i32.eqz
        br_if $B1
        local.get $l1
        i64.extend_i32_u
        local.set $l5
        loop $L2
          i32.const 65624
          i64.load
          i64.const 1
          i64.ne
          br_if $B0
          local.get $l0
          i32.const 65632
          i64.load
          local.tee $l2
          local.get $l4
          i64.add
          local.tee $l3
          i64.store
          local.get $l0
          i32.const 65640
          i64.load
          local.tee $l6
          local.get $l3
          local.get $l2
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l2
          i64.store offset=8
          local.get $l0
          i32.const 65648
          i64.load
          local.tee $l3
          local.get $l2
          local.get $l6
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l2
          i64.store offset=16
          local.get $l0
          i32.const 65656
          i64.load
          local.get $l2
          local.get $l3
          i64.lt_u
          i64.extend_i32_u
          i64.add
          i64.store offset=24
          local.get $l0
          call $env.ext_clear_storage
          local.get $l4
          i64.const 1
          i64.add
          local.tee $l2
          local.set $l4
          local.get $l2
          local.get $l5
          i64.ne
          br_if $L2
        end
      end
      local.get $l0
      i32.const 32
      i32.add
      global.set $g0
      return
    end
    unreachable)
  (func $f14 (type $t5)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32) (local $l6 i32) (local $l7 i64)
    block $B0
      block $B1
        i32.const 65664
        i32.load
        local.tee $l0
        i32.eqz
        br_if $B1
        i32.const 65672
        i32.load
        local.set $l4
        i32.const 65668
        i32.load
        local.set $l1
        loop $L2
          local.get $l1
          if $I3
            local.get $l1
            i32.const -1
            i32.add
            local.set $l1
            local.get $l0
            i32.load offset=96
            local.set $l0
            br $L2
          end
        end
        local.get $l4
        if $I4
          loop $L5
            local.get $l0
            i32.eqz
            br_if $B0
            local.get $l4
            i32.const -1
            i32.add
            local.set $l4
            block $B6
              local.get $l3
              local.get $l0
              i32.load16_u offset=6
              i32.lt_u
              if $I7
                local.get $l0
                local.get $l3
                i32.const 2
                i32.shl
                i32.add
                i32.const 52
                i32.add
                i32.load
                local.set $l1
                local.get $l3
                i32.const 1
                i32.add
                local.set $l3
                br $B6
              end
              loop $L8
                block $B9
                  local.get $l0
                  i32.load
                  local.tee $l2
                  i32.eqz
                  if $I10
                    i32.const 0
                    local.set $l2
                    br $B9
                  end
                  local.get $l1
                  i32.const 1
                  i32.add
                  local.set $l5
                  local.get $l0
                  i64.load16_u offset=4
                  i64.const 32
                  i64.shl
                  local.set $l7
                end
                local.get $l0
                i32.const 144
                i32.const 96
                local.get $l1
                select
                call $f19
                local.get $l5
                local.set $l1
                local.get $l7
                i64.const 32
                i64.shr_u
                i32.wrap_i64
                local.tee $l6
                local.get $l2
                local.tee $l0
                i32.load16_u offset=6
                i32.ge_u
                br_if $L8
              end
              local.get $l6
              i32.const 1
              i32.add
              local.set $l3
              local.get $l0
              local.get $l6
              i32.const 2
              i32.shl
              i32.add
              i32.const 52
              i32.add
              i32.load
              local.set $l1
              local.get $l5
              i32.eqz
              br_if $B6
              local.get $l0
              local.get $l3
              i32.const 2
              i32.shl
              i32.add
              i32.const 96
              i32.add
              i32.load
              local.set $l0
              i32.const 0
              local.set $l3
              local.get $l5
              i32.const 1
              i32.eq
              br_if $B6
              i32.const 1
              local.set $l2
              loop $L11
                local.get $l0
                i32.load offset=96
                local.set $l0
                local.get $l5
                local.get $l2
                i32.const 1
                i32.add
                local.tee $l2
                i32.ne
                br_if $L11
              end
            end
            local.get $l1
            i32.const 44
            call $f19
            i32.const 0
            local.set $l1
            local.get $l4
            br_if $L5
          end
        end
        local.get $l0
        i32.eqz
        br_if $B1
        local.get $l0
        i32.load
        local.set $l2
        local.get $l0
        i32.const 144
        i32.const 96
        local.get $l1
        select
        call $f19
        local.get $l2
        i32.eqz
        br_if $B1
        local.get $l1
        i32.const 1
        i32.add
        local.set $l1
        loop $L12
          local.get $l2
          i32.load
          local.set $l0
          local.get $l2
          i32.const 144
          i32.const 96
          local.get $l1
          select
          call $f19
          local.get $l1
          local.get $l0
          i32.const 0
          i32.ne
          i32.add
          local.set $l1
          local.get $l0
          local.tee $l2
          br_if $L12
        end
      end
      return
    end
    unreachable)
  (func $f15 (type $t5)
    (local $l0 i32) (local $l1 i32) (local $l2 i64) (local $l3 i64) (local $l4 i64) (local $l5 i64) (local $l6 i64)
    global.get $g0
    i32.const 32
    i32.sub
    local.tee $l0
    global.set $g0
    block $B0
      block $B1
        i32.const 65792
        i64.load
        i64.const 1
        i64.ne
        br_if $B1
        i32.const 65736
        call $f18
        i32.load
        local.tee $l1
        i32.eqz
        br_if $B1
        local.get $l1
        i64.extend_i32_u
        local.set $l5
        loop $L2
          i32.const 65792
          i64.load
          i64.const 1
          i64.ne
          br_if $B0
          local.get $l0
          i32.const 65800
          i64.load
          local.tee $l2
          local.get $l4
          i64.add
          local.tee $l3
          i64.store
          local.get $l0
          i32.const 65808
          i64.load
          local.tee $l6
          local.get $l3
          local.get $l2
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l2
          i64.store offset=8
          local.get $l0
          i32.const 65816
          i64.load
          local.tee $l3
          local.get $l2
          local.get $l6
          i64.lt_u
          i64.extend_i32_u
          i64.add
          local.tee $l2
          i64.store offset=16
          local.get $l0
          i32.const 65824
          i64.load
          local.get $l2
          local.get $l3
          i64.lt_u
          i64.extend_i32_u
          i64.add
          i64.store offset=24
          local.get $l0
          call $env.ext_clear_storage
          local.get $l4
          i64.const 1
          i64.add
          local.tee $l2
          local.set $l4
          local.get $l2
          local.get $l5
          i64.ne
          br_if $L2
        end
      end
      local.get $l0
      i32.const 32
      i32.add
      global.set $g0
      return
    end
    unreachable)
  (func $f16 (type $t5)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32) (local $l6 i32) (local $l7 i64)
    block $B0
      block $B1
        i32.const 65832
        i32.load
        local.tee $l0
        i32.eqz
        br_if $B1
        i32.const 65840
        i32.load
        local.set $l4
        i32.const 65836
        i32.load
        local.set $l1
        loop $L2
          local.get $l1
          if $I3
            local.get $l1
            i32.const -1
            i32.add
            local.set $l1
            local.get $l0
            i32.load offset=96
            local.set $l0
            br $L2
          end
        end
        local.get $l4
        if $I4
          loop $L5
            local.get $l0
            i32.eqz
            br_if $B0
            local.get $l4
            i32.const -1
            i32.add
            local.set $l4
            block $B6
              local.get $l3
              local.get $l0
              i32.load16_u offset=6
              i32.lt_u
              if $I7
                local.get $l0
                local.get $l3
                i32.const 2
                i32.shl
                i32.add
                i32.const 52
                i32.add
                i32.load
                local.set $l1
                local.get $l3
                i32.const 1
                i32.add
                local.set $l3
                br $B6
              end
              loop $L8
                block $B9
                  local.get $l0
                  i32.load
                  local.tee $l2
                  i32.eqz
                  if $I10
                    i32.const 0
                    local.set $l2
                    br $B9
                  end
                  local.get $l1
                  i32.const 1
                  i32.add
                  local.set $l5
                  local.get $l0
                  i64.load16_u offset=4
                  i64.const 32
                  i64.shl
                  local.set $l7
                end
                local.get $l0
                i32.const 144
                i32.const 96
                local.get $l1
                select
                call $f19
                local.get $l5
                local.set $l1
                local.get $l7
                i64.const 32
                i64.shr_u
                i32.wrap_i64
                local.tee $l6
                local.get $l2
                local.tee $l0
                i32.load16_u offset=6
                i32.ge_u
                br_if $L8
              end
              local.get $l6
              i32.const 1
              i32.add
              local.set $l3
              local.get $l0
              local.get $l6
              i32.const 2
              i32.shl
              i32.add
              i32.const 52
              i32.add
              i32.load
              local.set $l1
              local.get $l5
              i32.eqz
              br_if $B6
              local.get $l0
              local.get $l3
              i32.const 2
              i32.shl
              i32.add
              i32.const 96
              i32.add
              i32.load
              local.set $l0
              i32.const 0
              local.set $l3
              local.get $l5
              i32.const 1
              i32.eq
              br_if $B6
              i32.const 1
              local.set $l2
              loop $L11
                local.get $l0
                i32.load offset=96
                local.set $l0
                local.get $l5
                local.get $l2
                i32.const 1
                i32.add
                local.tee $l2
                i32.ne
                br_if $L11
              end
            end
            local.get $l1
            i32.const 48
            call $f19
            i32.const 0
            local.set $l1
            local.get $l4
            br_if $L5
          end
        end
        local.get $l0
        i32.eqz
        br_if $B1
        local.get $l0
        i32.load
        local.set $l2
        local.get $l0
        i32.const 144
        i32.const 96
        local.get $l1
        select
        call $f19
        local.get $l2
        i32.eqz
        br_if $B1
        local.get $l1
        i32.const 1
        i32.add
        local.set $l1
        loop $L12
          local.get $l2
          i32.load
          local.set $l0
          local.get $l2
          i32.const 144
          i32.const 96
          local.get $l1
          select
          call $f19
          local.get $l1
          local.get $l0
          i32.const 0
          i32.ne
          i32.add
          local.set $l1
          local.get $l0
          local.tee $l2
          br_if $L12
        end
      end
      return
    end
    unreachable)
  (func $f17 (type $t2) (param $p0 i32) (param $p1 i32)
    (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i64) (local $l6 i64) (local $l7 i64)
    global.get $g0
    i32.const 48
    i32.sub
    local.tee $l2
    global.set $g0
    block $B0
      local.get $p0
      i32.load offset=40
      i32.const 2
      i32.eq
      br_if $B0
      local.get $p0
      i32.const 48
      i32.add
      local.tee $l3
      i32.load8_u
      local.get $l3
      i32.const 1
      i32.store8
      i32.const 1
      i32.and
      br_if $B0
      local.get $p1
      i64.load offset=32
      local.set $l5
      local.get $p1
      i64.const 1
      i64.store offset=32
      local.get $p1
      local.get $l5
      local.get $p1
      i64.load
      local.tee $l6
      i64.add
      local.tee $l5
      i64.store
      local.get $p1
      local.get $p1
      i64.load offset=8
      local.tee $l7
      local.get $l5
      local.get $l6
      i64.lt_u
      i64.extend_i32_u
      i64.add
      local.tee $l5
      i64.store offset=8
      local.get $p1
      local.get $p1
      i64.load offset=16
      local.tee $l6
      local.get $l5
      local.get $l7
      i64.lt_u
      i64.extend_i32_u
      i64.add
      local.tee $l5
      i64.store offset=16
      local.get $p1
      local.get $p1
      i64.load offset=24
      local.get $l5
      local.get $l6
      i64.lt_u
      i64.extend_i32_u
      i64.add
      i64.store offset=24
      local.get $p0
      i32.load offset=40
      i32.const 1
      i32.ne
      if $I1
        local.get $l2
        i32.const 32
        i32.add
        local.get $p1
        i32.const 24
        i32.add
        i64.load
        i64.store
        local.get $l2
        i32.const 24
        i32.add
        local.get $p1
        i32.const 16
        i32.add
        i64.load
        i64.store
        local.get $l2
        i32.const 16
        i32.add
        local.get $p1
        i32.const 8
        i32.add
        i64.load
        i64.store
        local.get $l2
        i64.const 1
        i64.store offset=40
        local.get $l2
        local.get $p1
        i64.load
        i64.store offset=8
        local.get $l2
        i32.const 8
        i32.add
        call $env.ext_clear_storage
        br $B0
      end
      local.get $l2
      i32.const 32
      i32.add
      local.get $p1
      i32.const 24
      i32.add
      i64.load
      i64.store
      local.get $l2
      i32.const 24
      i32.add
      local.get $p1
      i32.const 16
      i32.add
      i64.load
      i64.store
      local.get $l2
      i32.const 16
      i32.add
      local.get $p1
      i32.const 8
      i32.add
      i64.load
      i64.store
      local.get $p1
      i64.load
      local.set $l5
      i32.const 65856
      local.get $p0
      i32.const 44
      i32.add
      i32.load
      i32.store
      i32.const 65852
      i32.const 4
      i32.store
      local.get $l2
      i64.const 1
      i64.store offset=40
      local.get $l2
      local.get $l5
      i64.store offset=8
      local.get $l2
      i32.const 8
      i32.add
      i32.const 65856
      i32.const 4
      call $env.ext_set_storage
    end
    local.get $l2
    i32.const 48
    i32.add
    global.set $g0)
  (func $f18 (type $t3) (param $p0 i32) (result i32)
    (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32)
    global.get $g0
    i32.const 112
    i32.sub
    local.tee $l1
    global.set $g0
    block $B0
      block $B1
        block $B2
          block $B3
            block $B4
              block $B5
                block $B6
                  local.get $p0
                  i32.load offset=40
                  local.tee $l3
                  i32.const 2
                  i32.eq
                  if $I7
                    local.get $l1
                    i32.const 16
                    i32.add
                    local.tee $l3
                    local.get $p0
                    i32.const 16
                    i32.add
                    i64.load
                    i64.store
                    local.get $l1
                    i32.const 24
                    i32.add
                    local.tee $l2
                    local.get $p0
                    i32.const 24
                    i32.add
                    i64.load
                    i64.store
                    local.get $l1
                    i32.const 32
                    i32.add
                    local.tee $l4
                    local.get $p0
                    i32.const 32
                    i32.add
                    i64.load
                    i64.store
                    local.get $l1
                    local.get $p0
                    i64.load offset=8
                    i64.store offset=8
                    local.get $p0
                    block $B8 (result i32)
                      i32.const 0
                      local.get $p0
                      i64.load
                      i64.const 1
                      i64.ne
                      br_if $B8
                      drop
                      local.get $l1
                      i32.const -64
                      i32.sub
                      local.get $l4
                      i64.load
                      i64.store
                      local.get $l1
                      i32.const 56
                      i32.add
                      local.get $l2
                      i64.load
                      i64.store
                      local.get $l1
                      i32.const 48
                      i32.add
                      local.get $l3
                      i64.load
                      i64.store
                      local.get $l1
                      local.get $l1
                      i64.load offset=8
                      i64.store offset=40
                      local.get $l1
                      i32.const 40
                      i32.add
                      call $env.ext_get_storage
                      local.tee $l3
                      i32.const 1
                      i32.gt_u
                      br_if $B4
                      i32.const 0
                      local.get $l3
                      i32.const 1
                      i32.sub
                      i32.eqz
                      br_if $B8
                      drop
                      call $env.ext_scratch_size
                      local.tee $l2
                      i32.const 16385
                      i32.ge_u
                      br_if $B4
                      i32.const 65852
                      local.get $l2
                      i32.store
                      i32.const 65856
                      i32.const 0
                      local.get $l2
                      call $env.ext_scratch_read
                      i32.const 65852
                      i32.load
                      local.tee $l3
                      i32.const 16385
                      i32.ge_u
                      br_if $B6
                      local.get $l3
                      local.get $l2
                      i32.lt_u
                      br_if $B5
                      local.get $l1
                      i32.const 96
                      i32.add
                      local.get $l1
                      i32.const -64
                      i32.sub
                      i64.load
                      i64.store
                      local.get $l1
                      i32.const 88
                      i32.add
                      local.get $l1
                      i32.const 56
                      i32.add
                      i64.load
                      i64.store
                      local.get $l1
                      i32.const 80
                      i32.add
                      local.get $l1
                      i32.const 48
                      i32.add
                      i64.load
                      i64.store
                      local.get $l1
                      local.get $l1
                      i64.load offset=40
                      i64.store offset=72
                      local.get $l1
                      i64.const 1
                      i64.store offset=104
                      local.get $l1
                      i32.const 72
                      i32.add
                      call $env.ext_get_storage
                      local.tee $l2
                      if $I9
                        local.get $l2
                        i32.const 1
                        i32.ne
                        br_if $B4
                        unreachable
                      end
                      call $env.ext_scratch_size
                      local.tee $l2
                      i32.const 16385
                      i32.ge_u
                      br_if $B4
                      i32.const 65852
                      local.get $l2
                      i32.store
                      i32.const 65856
                      i32.const 0
                      local.get $l2
                      call $env.ext_scratch_read
                      i32.const 65852
                      i32.load
                      local.tee $l3
                      i32.const 16385
                      i32.ge_u
                      br_if $B3
                      local.get $l3
                      local.get $l2
                      i32.lt_u
                      br_if $B2
                      local.get $l2
                      i32.const 4
                      i32.lt_u
                      br_if $B1
                      i32.const 65856
                      i32.load
                      local.set $l2
                      i32.const 1
                    end
                    local.tee $l3
                    i32.store offset=40
                    local.get $p0
                    i32.const 48
                    i32.add
                    i32.const 1
                    i32.store8
                    local.get $p0
                    i32.const 44
                    i32.add
                    local.get $l2
                    i32.store
                  end
                  local.get $l3
                  i32.const 1
                  i32.ne
                  br_if $B0
                  local.get $l1
                  i32.const 112
                  i32.add
                  global.set $g0
                  local.get $p0
                  i32.const 44
                  i32.add
                  return
                end
                unreachable
              end
              unreachable
            end
            unreachable
          end
          unreachable
        end
        unreachable
      end
      unreachable
    end
    unreachable)
  (func $f19 (type $t2) (param $p0 i32) (param $p1 i32)
    local.get $p0
    local.get $p1
    call $f12)
  (func $f20 (type $t6) (param $p0 i32)
    (local $l1 i32) (local $l2 i32)
    i32.const 272
    local.set $l2
    i32.const 65576
    local.set $l1
    loop $L0
      local.get $l1
      local.get $p0
      i32.load8_u
      i32.store8
      local.get $l1
      i32.const 1
      i32.add
      local.set $l1
      local.get $p0
      i32.const 1
      i32.add
      local.set $p0
      local.get $l2
      i32.const -1
      i32.add
      local.tee $l2
      br_if $L0
    end)
  (global $g0 (mut i32) (i32.const 65536))
  (export "deploy" (func $deploy))
  (export "call" (func $call))
  (data $d0 (i32.const 65536) "\05\00\00\00\06\00\00\00\07\00\00\00\00\00\00\00\01\00\00\00\02\00\00\00\03\00\00\00\04")
  (data $d1 (i32.const 65568) "\02"))
