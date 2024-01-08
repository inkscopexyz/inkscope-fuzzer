(module
  (type $t0 (func (param i32 i32)))
  (type $t1 (func))
  (type $t2 (func (param i32 i32 i32 i32) (result i32)))
  (type $t3 (func (param i32 i32 i32)))
  (type $t4 (func (result i32)))
  (type $t5 (func (param i32) (result i32)))
  (type $t6 (func (param i32)))
  (import "seal1" "get_storage" (func $seal1.get_storage (type $t2)))
  (import "seal0" "value_transferred" (func $seal0.value_transferred (type $t0)))
  (import "seal0" "input" (func $seal0.input (type $t0)))
  (import "seal2" "set_storage" (func $seal2.set_storage (type $t2)))
  (import "seal0" "seal_return" (func $seal0.seal_return (type $t3)))
  (import "env" "memory" (memory $env.memory 2 16))
  (func $f5 (type $t0) (param $p0 i32) (param $p1 i32)
    (local $l2 i32)
    global.get $g0
    i32.const 16
    i32.sub
    local.tee $l2
    global.set $g0
    local.get $l2
    local.get $p0
    i32.store8 offset=15
    local.get $p1
    local.get $l2
    i32.const 15
    i32.add
    i32.const 1
    call $f6
    local.get $l2
    i32.const 16
    i32.add
    global.set $g0)
  (func $f6 (type $t3) (param $p0 i32) (param $p1 i32) (param $p2 i32)
    (local $l3 i32) (local $l4 i32) (local $l5 i32)
    block $B0
      block $B1
        local.get $p0
        i32.load offset=8
        local.tee $l3
        local.get $p2
        i32.add
        local.tee $l4
        local.get $l3
        i32.lt_u
        br_if $B1
        local.get $l4
        local.get $p0
        i32.load offset=4
        i32.gt_u
        br_if $B1
        local.get $l4
        local.get $l3
        i32.sub
        local.get $p2
        i32.ne
        br_if $B0
        local.get $p0
        i32.load
        local.get $l3
        i32.add
        local.set $l5
        i32.const 0
        local.set $l3
        loop $L2 (result i32)
          local.get $p2
          local.get $l3
          i32.eq
          if $I3 (result i32)
            local.get $l5
          else
            local.get $l3
            local.get $l5
            i32.add
            local.get $p1
            local.get $l3
            i32.add
            i32.load8_u
            i32.store8
            local.get $l3
            i32.const 1
            i32.add
            local.set $l3
            br $L2
          end
        end
        drop
        local.get $p0
        local.get $l4
        i32.store offset=8
        return
      end
      unreachable
    end
    unreachable)
  (func $f7 (type $t4) (result i32)
    (local $l0 i32) (local $l1 i64) (local $l2 i64)
    global.get $g0
    i32.const 32
    i32.sub
    local.tee $l0
    global.set $g0
    local.get $l0
    i64.const 0
    i64.store offset=8
    local.get $l0
    i64.const 0
    i64.store
    local.get $l0
    i32.const 16
    i32.store offset=28
    local.get $l0
    local.get $l0
    i32.const 28
    i32.add
    call $seal0.value_transferred
    local.get $l0
    i64.load offset=8
    local.set $l1
    local.get $l0
    i64.load
    local.set $l2
    local.get $l0
    i32.const 32
    i32.add
    global.set $g0
    i32.const 5
    i32.const 4
    local.get $l1
    local.get $l2
    i64.or
    i64.eqz
    select)
  (func $f8 (type $t5) (param $p0 i32) (result i32)
    (local $l1 i32)
    i32.const 1
    i32.const 2
    local.get $p0
    i32.load offset=4
    local.tee $l1
    if $I0 (result i32)
      local.get $p0
      local.get $l1
      i32.const 1
      i32.sub
      i32.store offset=4
      local.get $p0
      local.get $p0
      i32.load
      local.tee $p0
      i32.const 1
      i32.add
      i32.store
      local.get $p0
      i32.load8_u
    else
      local.get $p0
    end
    i32.const 255
    i32.and
    local.tee $p0
    i32.const 1
    i32.eq
    select
    i32.const 0
    local.get $p0
    select
    i32.const 2
    local.get $l1
    select)
  (func $f9 (type $t1)
    i32.const 65536
    i32.const 0
    i32.store16
    i32.const 0
    i32.const 2
    call $f12
    unreachable)
  (func $f10 (type $t0) (param $p0 i32) (param $p1 i32)
    (local $l2 i32) (local $l3 i32)
    block $B0 (result i32)
      local.get $p1
      i32.eqz
      if $I1
        i32.const 65536
        local.set $p1
        i32.const 1
        br $B0
      end
      i32.const 1
      local.set $l2
      i32.const 65536
      i32.const 1
      i32.store8
      i32.const 65537
      local.set $p1
      i32.const 2
    end
    local.set $l3
    local.get $p1
    local.get $l2
    i32.store8
    local.get $p0
    local.get $l3
    call $f12
    unreachable)
  (func $f11 (type $t6) (param $p0 i32)
    (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32)
    global.get $g0
    i32.const 16
    i32.sub
    local.tee $l1
    global.set $g0
    local.get $l1
    i64.const 16384
    i64.store offset=8 align=4
    local.get $l1
    i32.const 65536
    i32.store offset=4
    local.get $l1
    i32.const 0
    i32.store
    local.get $l1
    i32.const 4
    i32.add
    local.get $l1
    i32.const 4
    call $f6
    block $B0
      local.get $l1
      i32.load offset=8
      local.tee $l4
      local.get $l1
      i32.load offset=12
      local.tee $l2
      i32.lt_u
      br_if $B0
      local.get $l1
      i32.load offset=4
      local.set $l3
      local.get $l1
      i32.const 0
      i32.store offset=12
      local.get $l1
      local.get $l4
      local.get $l2
      i32.sub
      i32.store offset=8
      local.get $l1
      local.get $l2
      local.get $l3
      i32.add
      i32.store offset=4
      local.get $p0
      local.get $l1
      i32.const 4
      i32.add
      call $f5
      local.get $l1
      i32.load offset=12
      local.tee $p0
      local.get $l1
      i32.load offset=8
      i32.gt_u
      br_if $B0
      local.get $l3
      local.get $l2
      local.get $l1
      i32.load offset=4
      local.get $p0
      call $seal2.set_storage
      drop
      local.get $l1
      i32.const 16
      i32.add
      global.set $g0
      return
    end
    unreachable)
  (func $f12 (type $t0) (param $p0 i32) (param $p1 i32)
    local.get $p0
    i32.const 65536
    local.get $p1
    call $seal0.seal_return
    unreachable)
  (func $call (type $t1)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32)
    global.get $g0
    i32.const 16
    i32.sub
    local.tee $l0
    global.set $g0
    block $B0
      block $B1
        block $B2
          call $f7
          i32.const 255
          i32.and
          i32.const 5
          i32.ne
          br_if $B2
          local.get $l0
          i32.const 16384
          i32.store
          i32.const 65536
          local.get $l0
          call $seal0.input
          local.get $l0
          i32.load
          local.tee $l1
          i32.const 16385
          i32.ge_u
          br_if $B2
          local.get $l1
          i32.const 4
          i32.lt_u
          br_if $B0
          i32.const 65539
          i32.load8_u
          local.set $l1
          i32.const 65538
          i32.load8_u
          local.set $l2
          i32.const 65537
          i32.load8_u
          local.set $l3
          block $B3
            i32.const 65536
            i32.load8_u
            local.tee $l4
            i32.const 47
            i32.ne
            if $I4
              local.get $l4
              i32.const 99
              i32.ne
              br_if $B0
              i32.const 1
              local.set $l5
              local.get $l3
              i32.const 58
              i32.ne
              local.get $l2
              i32.const 165
              i32.ne
              i32.or
              local.get $l1
              i32.const 81
              i32.ne
              i32.or
              i32.eqz
              br_if $B3
              br $B0
            end
            local.get $l3
            i32.const 134
            i32.ne
            local.get $l2
            i32.const 91
            i32.ne
            i32.or
            local.get $l1
            i32.const 217
            i32.ne
            i32.or
            br_if $B0
          end
          local.get $l0
          i64.const 16384
          i64.store offset=4 align=4
          local.get $l0
          i32.const 65536
          i32.store
          local.get $l0
          i32.const 0
          i32.store offset=12
          local.get $l0
          local.get $l0
          i32.const 12
          i32.add
          i32.const 4
          call $f6
          local.get $l0
          i32.load offset=4
          local.tee $l3
          local.get $l0
          i32.load offset=8
          local.tee $l1
          i32.lt_u
          br_if $B2
          local.get $l0
          i32.load
          local.set $l2
          local.get $l0
          local.get $l3
          local.get $l1
          i32.sub
          local.tee $l3
          i32.store
          local.get $l2
          local.get $l1
          local.get $l1
          local.get $l2
          i32.add
          local.tee $l1
          local.get $l0
          call $seal1.get_storage
          local.get $l3
          local.get $l0
          i32.load
          local.tee $l2
          i32.lt_u
          i32.or
          br_if $B2
          local.get $l0
          local.get $l2
          i32.store offset=4
          local.get $l0
          local.get $l1
          i32.store
          local.get $l0
          call $f8
          local.tee $l1
          i32.const 255
          i32.and
          i32.const 2
          i32.eq
          br_if $B2
          local.get $l5
          br_if $B1
          global.get $g0
          i32.const 16
          i32.sub
          local.tee $l0
          global.set $g0
          local.get $l0
          i32.const 65536
          i32.store offset=4
          i32.const 65536
          i32.const 0
          i32.store8
          local.get $l0
          i64.const 4294983680
          i64.store offset=8 align=4
          local.get $l1
          i32.const 255
          i32.and
          i32.const 0
          i32.ne
          local.get $l0
          i32.const 4
          i32.add
          call $f5
          local.get $l0
          i32.load offset=12
          local.tee $l0
          i32.const 16385
          i32.ge_u
          if $I5
            unreachable
          end
          i32.const 0
          local.get $l0
          call $f12
          unreachable
        end
        unreachable
      end
      local.get $l1
      i32.const 255
      i32.and
      i32.eqz
      call $f11
      i32.const 0
      i32.const 0
      call $f10
      unreachable
    end
    i32.const 1
    i32.const 1
    call $f10
    unreachable)
  (func $deploy (type $t1)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32)
    global.get $g0
    i32.const 16
    i32.sub
    local.tee $l0
    global.set $g0
    block $B0
      block $B1
        call $f7
        i32.const 255
        i32.and
        i32.const 5
        i32.ne
        br_if $B1
        local.get $l0
        i32.const 16384
        i32.store offset=12
        i32.const 65536
        local.get $l0
        i32.const 12
        i32.add
        call $seal0.input
        local.get $l0
        i32.load offset=12
        local.tee $l1
        i32.const 16385
        i32.ge_u
        br_if $B1
        block $B2
          local.get $l1
          i32.const 4
          i32.lt_u
          br_if $B2
          local.get $l0
          i32.const 65540
          i32.store offset=4
          local.get $l0
          local.get $l1
          i32.const 4
          i32.sub
          i32.store offset=8
          i32.const 65539
          i32.load8_u
          local.set $l1
          i32.const 65538
          i32.load8_u
          local.set $l2
          i32.const 65537
          i32.load8_u
          local.set $l3
          i32.const 65536
          i32.load8_u
          local.tee $l4
          i32.const 237
          i32.ne
          if $I3
            local.get $l4
            i32.const 155
            i32.ne
            local.get $l3
            i32.const 174
            i32.ne
            i32.or
            local.get $l2
            i32.const 157
            i32.ne
            local.get $l1
            i32.const 94
            i32.ne
            i32.or
            i32.or
            br_if $B2
            local.get $l0
            i32.const 4
            i32.add
            call $f8
            local.tee $l0
            i32.const 255
            i32.and
            i32.const 2
            i32.eq
            br_if $B2
            local.get $l0
            call $f11
            call $f9
            unreachable
          end
          local.get $l3
          i32.const 75
          i32.ne
          local.get $l2
          i32.const 157
          i32.ne
          i32.or
          br_if $B2
          local.get $l1
          i32.const 27
          i32.eq
          br_if $B0
        end
        i32.const 1
        i32.const 1
        call $f10
        unreachable
      end
      unreachable
    end
    i32.const 0
    call $f11
    call $f9
    unreachable)
  (global $g0 (mut i32) (i32.const 65536))
  (export "call" (func $call))
  (export "deploy" (func $deploy)))
