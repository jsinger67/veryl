module veryl_testcase_Module18;
    logic [20-1:0] a;
    logic          b;
    logic          c; always_comb c = 1;

    always_comb a = {a[10:0], c};
    always_comb b = {{10{a[10:0]}}, {4{c}}};

    // multi-line concatenation
    logic _d; always_comb _d = {
        {8{a}}, {8{b}}, {8{c}},
        {8{a}}, {8{b}}, {8{c}},
        {8{a}}, {8{b}}, {8{c}},
        {8{a}}, {8{b}}, {8{c}},
        {8{a}}, {8{b}}, {8{c}},
        {8{a}}, {8{b}}, {8{c}},
        {8{a}}, {8{b}}, {8{c}}
    };
endmodule
//# sourceMappingURL=../map/testcases/sv/18_concatenation.sv.map
